// RPC client for the wasm worker in src/worker.ts.

type Pending = { resolve: (v: unknown) => void; reject: (e: Error) => void };

let worker: Worker | null = null;
let readyResolve: (ok: boolean) => void;
let readyReject: (e: Error) => void;
const readyPromise = new Promise<boolean>((res, rej) => {
  readyResolve = res;
  readyReject = rej;
});

let nextId = 0;
const pending = new Map<number, Pending>();

export function startWorker(onReady: (err: string | null) => void) {
  worker = new Worker(new URL("../worker.ts", import.meta.url), {
    type: "module",
  });
  worker.onmessage = (event: MessageEvent) => {
    const { id, ready, ok, result, error } = event.data;
    if (ready !== undefined) {
      if (ready) {
        readyResolve(true);
        onReady(null);
      } else {
        const e = new Error(error ?? "wasm init failed");
        readyReject(e);
        onReady(e.message);
      }
      return;
    }
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    if (ok) p.resolve(result);
    else p.reject(new Error(error));
  };
  worker.onerror = (e) => {
    const err = new Error(e.message || "worker error");
    readyReject(err);
    for (const p of pending.values()) p.reject(err);
    pending.clear();
  };
}

export function call<T = unknown>(fn: string, ...args: unknown[]): Promise<T> {
  if (!worker) return Promise.reject(new Error("worker not started"));
  nextId += 1;
  return new Promise<T>((resolve, reject) => {
    pending.set(nextId, {
      resolve: resolve as (v: unknown) => void,
      reject,
    });
    worker!.postMessage({ id: nextId, fn, args });
  });
}

export const wasmReady = readyPromise;
