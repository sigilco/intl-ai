# This formula is generated and updated by the intl-ai release workflow.
# Do not edit manually; changes will be overwritten.

class IntlAi < Formula
  desc "AI-powered build-time i18n translation CLI"
  homepage "https://intl-ai.pages.dev"
  license "MIT"
  version "VERSION_PLACEHOLDER"

  on_macos do
    on_arm do
      url "https://github.com/sigilco/intl-ai/releases/download/v#{version}/intl-ai-bun-darwin-arm64"
      sha256 "SHA256_DARWIN_ARM64" # populated by CI on release
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/sigilco/intl-ai/releases/download/v#{version}/intl-ai-bun-linux-x64"
      sha256 "SHA256_LINUX_X64" # populated by CI on release
    end

    on_arm do
      url "https://github.com/sigilco/intl-ai/releases/download/v#{version}/intl-ai-bun-linux-arm64"
      sha256 "SHA256_LINUX_ARM64" # populated by CI on release
    end
  end

  def install
    if Hardware::CPU.arm? && OS.mac?
      bin.install "intl-ai-bun-darwin-arm64" => "intl-ai"
    elsif Hardware::CPU.intel? && OS.linux?
      bin.install "intl-ai-bun-linux-x64" => "intl-ai"
    elsif Hardware::CPU.arm? && OS.linux?
      bin.install "intl-ai-bun-linux-arm64" => "intl-ai"
    end
  end

  test do
    system "#{bin}/intl-ai", "--version"
  end
end
