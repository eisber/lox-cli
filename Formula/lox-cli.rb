class LoxCli < Formula
  desc "AI agent tooling for Loxone Miniserver — config-as-code CLI"
  homepage "https://github.com/eisber/lox-cli"
  version "0.12.0"
  license "AGPL-3.0-or-later"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-macos-aarch64"
      sha256 "98a520da9a61c77b035f7f47963c208bd99884b54b83c5aa4e511db13f64d0e5"
    else
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-macos-x86_64"
      sha256 "aba4119722778c82766731221ff65dc4d034b70e89d1b4027dbf96cdcfcb8e25"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-linux-aarch64"
      sha256 "38b3f3a6498fc20621babb522d22df5240e9b148449ba55651ef9fda400544a4"
    else
      url "https://github.com/eisber/lox-cli/releases/download/v#{version}/lox-cli-linux-x86_64"
      sha256 "39a5c708b0a14f6c5762eca789d3f3169b085c291f1cf269aa743bfd0eb4e63d"
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
