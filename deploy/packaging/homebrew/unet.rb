class Unet < Formula
  desc "Network configuration management and automation tool"
  homepage "https://github.com/example/unet"
  url "https://github.com/example/unet/archive/v0.1.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"  # Update with actual SHA256
  license "MIT"
  head "https://github.com/example/unet.git", branch: "main"

  livecheck do
    url :stable
    regex(/^v?(\d+(?:\.\d+)+)$/i)
  end

  depends_on "rust" => :build
  depends_on "pkg-config" => :build
  depends_on "openssl@3"
  depends_on "sqlite"
  depends_on "postgresql@15" => :optional

  def install
    # Set environment variables for build
    ENV["OPENSSL_DIR"] = Formula["openssl@3"].opt_prefix
    ENV["OPENSSL_NO_VENDOR"] = "1"
    ENV["PKG_CONFIG_PATH"] = "#{Formula["openssl@3"].opt_lib}/pkgconfig"

    # Build the workspace
    system "cargo", "build", "--release", "--workspace", "--bins"

    # Install binaries
    bin.install "target/release/unet"
    bin.install "target/release/unet-server"

    # Install configuration templates
    etc.install "config.toml" => "unet/config.toml.example"
    etc.install "config-postgres.toml" => "unet/config-postgres.toml.example" if build.with?("postgresql")
    etc.install "config-load-balancer.toml" => "unet/config-load-balancer.toml.example"

    # Install example policies and templates
    share.install "policies" => "unet/policies" if Dir.exist?("policies")
    share.install "templates" => "unet/templates" if Dir.exist?("templates")

    # Install documentation
    doc.install "README.md"
    doc.install "docs/book" => "html" if Dir.exist?("docs/book")

    # Create directories for data and logs
    (var/"lib/unet").mkpath
    (var/"log/unet").mkpath

    # Install launchd plist for macOS service management
    (buildpath/"homebrew.mxcl.unet-server.plist").write server_plist
    prefix.install "homebrew.mxcl.unet-server.plist"
  end

  def post_install
    # Set up default configuration if none exists
    config_file = etc/"unet/config.toml"
    example_file = etc/"unet/config.toml.example"
    
    unless config_file.exist?
      if example_file.exist?
        cp example_file, config_file
        ohai "Default configuration created at #{config_file}"
        ohai "Please review and customize the configuration before starting the service."
      end
    end

    # Set proper permissions
    (var/"lib/unet").chmod 0755
    (var/"log/unet").chmod 0755
  end

  def server_plist
    <<~EOS
      <?xml version="1.0" encoding="UTF-8"?>
      <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
      <plist version="1.0">
        <dict>
          <key>Label</key>
          <string>#{plist_name}</string>
          <key>ProgramArguments</key>
          <array>
            <string>#{opt_bin}/unet-server</string>
            <string>--config</string>
            <string>#{etc}/unet/config.toml</string>
          </array>
          <key>WorkingDirectory</key>
          <string>#{var}/lib/unet</string>
          <key>StandardOutPath</key>
          <string>#{var}/log/unet/unet-server.log</string>
          <key>StandardErrorPath</key>
          <string>#{var}/log/unet/unet-server.log</string>
          <key>EnvironmentVariables</key>
          <dict>
            <key>RUST_LOG</key>
            <string>info</string>
            <key>UNET_CONFIG_DIR</key>
            <string>#{etc}/unet</string>
            <key>UNET_DATA_DIR</key>
            <string>#{var}/lib/unet</string>
            <key>UNET_LOG_DIR</key>
            <string>#{var}/log/unet</string>
          </dict>
          <key>RunAtLoad</key>
          <false/>
          <key>KeepAlive</key>
          <dict>
            <key>SuccessfulExit</key>
            <false/>
          </dict>
          <key>ThrottleInterval</key>
          <integer>5</integer>
        </dict>
      </plist>
    EOS
  end

  service do
    run [opt_bin/"unet-server", "--config", etc/"unet/config.toml"]
    working_dir var/"lib/unet"
    log_path var/"log/unet/unet-server.log"
    error_log_path var/"log/unet/unet-server.log"
    environment_variables RUST_LOG: "info", UNET_CONFIG_DIR: etc/"unet", UNET_DATA_DIR: var/"lib/unet", UNET_LOG_DIR: var/"log/unet"
    keep_alive succesful_exit: false
  end

  test do
    # Test that the binaries were installed correctly
    assert_match "unet", shell_output("#{bin}/unet --version")
    assert_match "unet-server", shell_output("#{bin}/unet-server --version")

    # Test configuration file parsing
    (testpath/"test-config.toml").write <<~EOS
      [server]
      host = "127.0.0.1"
      port = 8080

      [database]
      url = "sqlite://test.db"

      [logging]
      level = "info"
    EOS

    # Test that the CLI can validate the config (dry run)
    system bin/"unet", "config", "validate", "--config", testpath/"test-config.toml"
  end
end