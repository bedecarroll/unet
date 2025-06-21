# 08 Deployment – Linux, Docker Compose & Nix‑Flake Guides

> **Audience:** Ops‑minded junior engineers tasked with deploying μNet in greenfield environments.
>
> **Objective:** Provide **end‑to‑end**, copy‑paste‑friendly instructions for three deployment paths:
>
> 1. **Bare‑metal / VM** using **systemd** (SQLite by default)
> 2. **Docker Compose** for container‑centric shops
> 3. A forward‑looking **Nix flake** outline (for infra‑as‑code adopters)
>
> Along the way we cover sizing, security, backups, upgrades, and rollback.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Directory Layout Convention](#2-directory-layout-convention)
3. [Option A – Bare‑Metal (systemd)](#3-option-a--bare-metal-systemd)\
   3.1 [Install Binaries](#31-install-binaries)\
   3.2 [Create Service Account](#32-create-service-account)\
   3.3 [Initial ](#33-initial-configtoml)[`config.toml`](#33-initial-configtoml)\
   3.4 [Systemd Unit](#34-systemd-unit)\
   3.5 [Start & Verify](#35-start--verify)\
   3.6 [Upgrades & Rollback](#36-upgrades--rollback)
4. [Option B – Docker Compose](#4-option-b--docker-compose)\
   4.1 [Compose File](#41-compose-file)\
   4.2 [Volumes & Secrets](#42-volumes--secrets)\
   4.3 [Zero‑Downtime Upgrade](#43-zero-downtime-upgrade)
5. [Option C – Nix Flake (Preview)](#5-option-c--nix-flake-preview)
6. [Sizing & Capacity Planning](#6-sizing--capacity-planning)
7. [Security Hardening](#7-security-hardening)
8. [Backups & Disaster Recovery](#8-backups--disaster-recovery)
9. [Monitoring & Logs](#9-monitoring--logs)
10. [Common Pitfalls](#10-common-pitfalls)

---

## 1  Prerequisites

| Component                    | Minimum Version                       | Notes                               |
| ---------------------------- | ------------------------------------- | ----------------------------------- |
| Linux distro                 | Ubuntu 20.04 LTS / Debian 11 / RHEL 8 | Systemd ≥ 245 required              |
| CPU / RAM                    | 2 vCPU / 2 GB                         | +1 GB per 10 k nodes polled         |
| Disk                         | 2 GB free                             | SQLite db + Git repos + logs        |
| Open ports                   | 8080/tcp (HTTP)                       | 8443/tcp if TLS enabled             |
| Outbound internet (optional) | 443/tcp                               | To clone template/policy Git repos  |
| Rust toolchain (dev only)    | 1.77 (stable)                         | Needed only if building from source |

> **Tip:** For PoC, a 1‑CPU micro VM (DigitalOcean 1 vCPU/1 GB) handles < 1 k nodes.

---

## 2  Directory Layout Convention

| Path             | Purpose                                     | Owner       | Back up?               |
| ---------------- | ------------------------------------------- | ----------- | ---------------------- |
| `/opt/unet/bin/` | Static binaries (`unet-server`, `unet-cli`) |  root\:root | No (reinstall)         |
| `/etc/unet/`     | `config.toml` + TLS certs                   | root\:unet  | Yes                    |
| `/var/lib/unet/` | SQLite database, Git clones                 | unet\:unet  | **Yes**                |
| `/var/log/unet/` | Rotated server logs                         | unet\:unet  | Optional (centralised) |

Adjust to distro standards if needed (`/usr/local/bin`, `/srv/unet`, etc.).

---

## 3  Option A – Bare‑Metal (systemd)

### 3.1 Install Binaries

```bash
# 1  Create download staging dir
sudo mkdir -p /opt/unet/bin && cd /opt/unet/bin

# 2  Fetch latest release (replace X.Y.Z)
curl -L -O https://github.com/your‑org/unet/releases/download/vX.Y.Z/unet-server-x86_64-unknown-linux-musl.tar.gz
curl -L -O https://github.com/your‑org/unet/releases/download/vX.Y.Z/unet-cli-x86_64-unknown-linux-musl.tar.gz

# 3  Verify checksum (SHA256)
sha256sum -c unet-server-x86_64-unknown-linux-musl.tar.gz.sha256

# 4  Extract & chmod
sudo tar -xzf unet-server-*.tar.gz --strip-components=1
sudo tar -xzf unet-cli-*.tar.gz --strip-components=1
sudo chmod 755 unet-server unet-cli
```

> **Note:** Binaries are static (MUSL) – no extra libs required.

### 3.2 Create Service Account

```bash
sudo useradd --system --create-home --home /var/lib/unet --shell /usr/sbin/nologin unet
sudo chown -R unet:unet /var/lib/unet
```

### 3.3 Initial `config.toml`

```bash
sudo install -d -m 750 -o unet -g unet /etc/unet
sudo nano /etc/unet/config.toml
```

Minimal starter:

```toml
[server]
bind = "0.0.0.0:8080"

[database]
type = "sqlite"
path = "/var/lib/unet/unet.db"

[git]
policies  = "https://github.com/org/unet-policies.git"
templates = "https://github.com/org/unet-templates.git"
sync_cron = "*/10 * * * *"
```

Copy SSH deploy keys into `/etc/unet/ssh/` if using private repos and set:

```toml
[git]
ssh_key_path = "/etc/unet/ssh/id_ed25519"
```

### 3.4 Systemd Unit

`sudo nano /etc/systemd/system/unet-server.service`

```ini
[Unit]
Description=μNet Server
After=network.target

[Service]
Type=simple
User=unet
Group=unet
WorkingDirectory=/var/lib/unet
ExecStart=/opt/unet/bin/unet-server --config /etc/unet/config.toml
Restart=on-failure
RestartSec=5s
# Security hardening
AmbientCapabilities=CAP_NET_BIND_SERVICE
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

Reload & enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now unet-server.service
```

### 3.5 Start & Verify

```bash
sudo systemctl status unet-server -n 50
curl -s http://localhost:8080/health/live | jq
```

Expect `{"status":"live"}`.

### 3.6 Upgrades & Rollback

```bash
# Download new tarball (vX.Y+1) into /opt/unet/bin.new
sudo systemctl stop unet-server
sudo mv /opt/unet/bin /opt/unet/bin.old
sudo mv /opt/unet/bin.new /opt/unet/bin
sudo systemctl start unet-server
# Verify health, then purge old
sudo rm -rf /opt/unet/bin.old
```

Rollback = reverse the `mv` steps.

---

## 4  Option B – Docker Compose

### 4.1 Compose File

`docker/docker-compose.yml`:

```yaml
version: "3.9"
services:
  unet-server:
    image: ghcr.io/your-org/unet-server:nightly
    container_name: unet-server
    user: 1000:1000           # run as non‑root UID 1000 inside container
    volumes:
      - ./data:/var/lib/unet
      - ./config:/etc/unet:ro
    environment:
      - RUST_LOG=info
    ports:
      - "8080:8080"
    restart: unless-stopped

  # optional: reverse‑proxy for TLS
  caddy:
    image: caddy:2
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
    ports:
      - "443:443"
    restart: unless-stopped
```

Directory structure:

```
docker/
├── data/                    # persists SQLite + Git repos
├── config/config.toml       # bind‑mounted read‑only
└── docker-compose.yml
```

### 4.2 Volumes & Secrets

- Mount SSH keys at `./config/ssh/` and `chmod 600`.
- To externalise SQLite, mount a named volume instead of local path.

### 4.3 Zero‑Downtime Upgrade

```bash
# Pull latest tag
sudo docker compose pull unet-server
# Recreate container
sudo docker compose up -d --no-deps unet-server
# Verify health endpoint, then prune old images
sudo docker image prune -f
```

Compose keeps the old container until new one healthy if `restart: unless‑stopped` set.

---

## 5  Option C – Nix Flake (Preview)

`flake.nix` sketch:

```nix
{
  description = "μNet network‑config platform";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in {
    packages.${system}.unet-server = pkgs.stdenv.mkDerivation {
      pname = "unet-server";
      version = "0.1.0";
      src = self;
      nativeBuildInputs = [ pkgs.rustPlatform.rust.cargo ];
      buildPhase = "cargo build --release --bin unet-server";
      installPhase = ''install -Dm755 target/release/unet-server $out/bin/unet-server'';
    };

    nixosModules.unet = { config, lib, pkgs, ... }: {
      services.unet = {
        enable = true;
        package = self.packages.${system}.unet-server;
        configFile = "/etc/unet/config.toml";
      };
    };
  };
}
```

Future work: submit to **Nixpkgs** for `nix profile install`. Juniors can run `nixos-rebuild switch --flake .#hostname`.

---

## 6  Sizing & Capacity Planning

| Nodes  | Poll Interval | CPU (vCPU) | RAM  | Notes                              |
| ------ | ------------- | ---------- | ---- | ---------------------------------- |
| < 1 k  | 15 min        | 1          | 1 GB | PoC; single‑board computer OK      |
| 1–5 k  | 10 min        | 2          | 2 GB | Template+policy eval < 300 ms      |
| 5–10 k | 5 min         | 4          | 4 GB | Consider Postgres back‑end         |
| >10 k  | 5 min         | 4+         | 8 GB | Split poller (future microservice) |

Disk: 1 GB per 10 k nodes/year assuming 50 bytes / poll row in history table (if enabled).

---

## 7  Security Hardening

1. **Run as non‑root** – service account or UID 1000 in container.
2. **TLS** – terminate with Caddy/Nginx or Axum’s `rustls` feature (`cert.pem`, `key.pem`).
3. **Firewall** – allow 8080/8443 only from jump hosts; outbound 161/UDP for SNMP from server.
4. **Secrets** – store Git SSH keys and JWT signing keys under `/etc/unet/` (600 permissions).
5. `` – disable kernel modules if containerised; enable `fs.protected_regular=1`.

---

## 8  Backups & Disaster Recovery

| Component | Method                                            | Frequency | Retention |
| --------- | ------------------------------------------------- | --------- | --------- |
| SQLite DB | `sqlite3 unet.db .backup /backup/unet-$(date).db` | hourly    | 24 h      |
| Git repos | Re‑cloneable – skip                               | ‑         | ‑         |
| Config    | Version‑controlled – skip                         | ‑         | ‑         |
| Templates | Same as above                                     | ‑         | ‑         |

> **Restore:** Stop service, copy snapshot over `unet.db`, `VACUUM`, start service.

Plan to move to Postgres + WAL shipping when node count > 50 k.

---

## 9  Monitoring & Logs

| Signal        | Path / Endpoint                 | Tooling                     |
| ------------- | ------------------------------- | --------------------------- |
| Health probe  | `/health/live`, `/health/ready` | `curl`, Prometheus blackbox |
| Metrics       | `/metrics` (Prometheus format)  | `prometheus`, `grafana`     |
| Logs          | stdout/stderr (systemd journal) | `journalctl -u unet-server` |
| SNMP failures | `poll_failure_total` metric     | Grafana alert               |

Add `-e OTEL_EXPORTER_OTLP_ENDPOINT` to container env to forward traces.

---

## 10  Common Pitfalls

| Symptom                          | Cause / Fix                                                                 |
| -------------------------------- | --------------------------------------------------------------------------- |
| `database locked` errors         | SQLite under heavy write load → set `PRAGMA busy_timeout=5000` in config.   |
| Git sync fails, server keeps old | Check SSH deploy key permissions (`chmod 600`); ensure `known_hosts` entry. |
| **503** at `/health/ready`       | Migrations pending – run `unet db migrate up` or check file permissions.    |
| SNMP poller exits with `timeout` | Firewall blocks UDP 161; set `snmp.retry=3` and verify route.               |
| Diff shows whole config changed  | Missing / incorrect `template-match` header – add more specific regex.      |

---

### Next Steps

1. Pick deployment path (A or B).
2. Allocate VM / container resources per §6.
3. Follow relevant section step‑by‑step.
4. After first boot, create an **admin** node via `unet-cli` and verify policy evaluation.

*Proceed to *[*09\_config\_match\_tool.md*](09_config_match_tool.md)* when ready.*

