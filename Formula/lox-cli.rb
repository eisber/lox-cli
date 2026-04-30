class LoxCli < Formula
  desc "AI agent tooling for Loxone Miniserver — config-as-code CLI"
  homepage "https://github.com/eisber/lox-cli"
  version "0.12.0"
  license "AGPL-3.0-or-later"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-macos-aarch64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-macos-x86_64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-linux-aarch64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-linux-x86_64"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    binary = Dir["lox-cli-*"].first || "lox"
    bin.install binary => "lox"
  end

  test do
    assert_match "lox #{version}", shell_output("#{bin}/lox --version")
  end
end
