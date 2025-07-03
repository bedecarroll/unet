# Metrics & Monitoring Guide

> **Audience:** Operators and SRE teams.
> **Objective:** Collect runtime metrics and integrate μNet with monitoring systems.

μNet exports Prometheus metrics at `/metrics` via the Axum Prometheus middleware. Typical metrics include HTTP request counts, SNMP polling failures, and policy evaluation durations.

## Enabling Metrics

Metrics are enabled by default. Point your Prometheus server at the μNet backend and configure Grafana dashboards for visualization.

## Custom Counters

The `MetricsManager` in `unet-core` exposes counters such as `snmp_poll_failure_total` and `policy_violation_total`.

## See Also

- [Server Backend](06_server_backend.md#8--logging-tracing--metrics)
- [Operational Runbooks](runbooks/README.md)
