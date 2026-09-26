<script setup lang="ts">
import { URLs, type UrlKey } from "../urls";

defineProps<{
  name: keyof typeof URLs;
  label?: string;
}>();
</script>

<template>
  <a :href="URLs[name]">{{ label ?? URLs[name] }}</a>
</template>
