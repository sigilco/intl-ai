// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "IntlAi",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15),
    ],
    products: [
        .library(name: "IntlAi", targets: ["IntlAi"])
    ],
    targets: [
        // Rust core as a static-library XCFramework; build it with
        // scripts/build-xcframework.sh. Release builds swap this local
        // path for a binaryTarget(url:checksum:) pointing at the
        // GitHub release asset.
        .binaryTarget(name: "IntlAiFFI", path: "IntlAiFFI.xcframework"),
        .target(
            name: "IntlAi",
            dependencies: ["IntlAiFFI"]
        ),
    ]
)
