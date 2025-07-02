{{/*
Expand the name of the chart.
*/}}
{{- define "unet.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "unet.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "unet.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "unet.labels" -}}
helm.sh/chart: {{ include "unet.chart" . }}
{{ include "unet.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: unet
{{- end }}

{{/*
Selector labels
*/}}
{{- define "unet.selectorLabels" -}}
app.kubernetes.io/name: {{ include "unet.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "unet.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "unet.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the secret to use
*/}}
{{- define "unet.secretName" -}}
{{- if .Values.secrets.external.enabled }}
{{- .Values.secrets.external.secretStore }}
{{- else }}
{{- include "unet.fullname" . }}-secrets
{{- end }}
{{- end }}

{{/*
Database URL for PostgreSQL
*/}}
{{- define "unet.databaseUrl" -}}
{{- if .Values.postgresql.enabled }}
postgres://{{ .Values.postgresql.auth.username }}:{{ .Values.postgresql.auth.password }}@{{ include "unet.fullname" . }}-postgresql:5432/{{ .Values.postgresql.auth.database }}
{{- else }}
sqlite:///app/data/unet.db
{{- end }}
{{- end }}

{{/*
Redis URL
*/}}
{{- define "unet.redisUrl" -}}
{{- if .Values.redis.enabled }}
redis://{{ include "unet.fullname" . }}-redis-master:6379
{{- else }}
""
{{- end }}
{{- end }}

{{/*
Environment-specific overrides
*/}}
{{- define "unet.environment" -}}
{{- $env := .Values.environment | default "default" }}
{{- if hasKey .Values.environments $env }}
{{- $envConfig := index .Values.environments $env }}
{{- $_ := mergeOverwrite .Values.unet $envConfig }}
{{- end }}
{{- end }}