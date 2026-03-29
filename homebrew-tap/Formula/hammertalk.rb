class Hammertalk < Formula
  desc "Push-to-talk transcription daemon with multiple engine support"
  homepage "https://github.com/hiasinho/hammertalk"
  url "https://github.com/hiasinho/hammertalk/archive/refs/tags/v1.2.0.tar.gz"
  sha256 "953bf0e466aff1c0bf150d14db8d7c3cef519528a44f0dd41a6b49e4a919b428"
  license "MIT"
  head "https://github.com/hiasinho/hammertalk.git", branch: "master"

  depends_on "cmake" => :build
  depends_on "rust" => :build

  on_linux do
    depends_on "ydotool"
  end

  def install
    if OS.mac?
      system "cargo", "build", "--release", "--features", "hotkey"
    else
      system "cargo", "build", "--release"
    end

    bin.install "target/release/hammertalk"
    bin.install "hammertalk-ctl"
    bin.install "download-model.sh" => "hammertalk-download-model"

    if OS.mac?
      # Install launchd plist template
      prefix.install "com.hammertalk.daemon.plist"
    else
      # Install systemd service template
      prefix.install "hammertalk.service"
    end
  end

  def post_install
    # Create model directory
    (var/"hammertalk/models").mkpath
  end

  def caveats
    s = <<~EOS
      Download a transcription model before first use:
        hammertalk-download-model                       # default: parakeet-tdt-v3-int8 (~640MB)
        hammertalk-download-model parakeet-tdt-v3-int8  # smaller int8 variant (~640MB)

    EOS

    if OS.mac?
      s += <<~EOS
        Grant permissions in System Settings → Privacy & Security:
          • Microphone
          • Accessibility (for text input and hotkey)

        Default hotkey: Fn (globe key). Override with:
          hammertalk --hotkey "Cmd+Shift+T"

        To start as a background service:
          brew services start hammertalk

        Or use launchd directly:
          cp #{opt_prefix}/com.hammertalk.daemon.plist ~/Library/LaunchAgents/
          # Edit the plist to set paths, then:
          launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.hammertalk.daemon.plist
      EOS
    else
      s += <<~EOS
        Ensure ydotoold is running:
          sudo systemctl enable --now ydotool

        To start as a background service:
          brew services start hammertalk
      EOS
    end

    s
  end

  service do
    run [opt_bin/"hammertalk"]
    keep_alive crashed: true
    log_path var/"log/hammertalk.log"
    error_log_path var/"log/hammertalk.log"
    environment_variables RUST_LOG: "info"
  end

  test do
    # hammertalk exits with error if no model is downloaded, but confirms it starts
    output = shell_output("#{bin}/hammertalk 2>&1", 1)
    assert_match "Hammertalk starting", output
  end
end
