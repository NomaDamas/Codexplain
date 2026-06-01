class Codexplain < Formula
  desc "Readable terminal explanation UX layer for Codex responses"
  homepage "https://github.com/NomaDamas/Codexplain"
  url "https://github.com/NomaDamas/Codexplain.git",
      tag: "v0.18.16"
  license :cannot_represent
  head "https://github.com/NomaDamas/Codexplain.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/codexplain"
    bin.install_symlink "codexplain" => "codexplain-codex"
    bin.install_symlink "codexplain" => "claudex"
    bin.install_symlink "codexplain" => "claudex-codex"
  end

  test do
    assert_match "Usage:", shell_output("#{bin}/codexplain --help")
    assert_match "contract=codexplain.quality-check.v1",
      shell_output("#{bin}/codexplain quality-check --width 88")
  end
end
