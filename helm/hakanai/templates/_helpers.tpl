{{/*
Expand the name of the chart.
*/}}
{{- define "hakanai.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "hakanai.fullname" -}}
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
{{- define "hakanai.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "hakanai.labels" -}}
helm.sh/chart: {{ include "hakanai.chart" . }}
{{ include "hakanai.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "hakanai.selectorLabels" -}}
app.kubernetes.io/name: {{ include "hakanai.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "hakanai.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "hakanai.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Redis service name
*/}}
{{- define "hakanai.redisHost" -}}
{{- if .Values.redis.enabled }}
{{- if eq .Values.redis.architecture "replication" }}
{{- printf "%s-redis-master" .Release.Name }}
{{- else }}
{{- printf "%s-redis-master" .Release.Name }}
{{- end }}
{{- else }}
{{- required "External Redis host must be provided when redis.enabled is false" .Values.externalRedis.host }}
{{- end }}
{{- end }}

{{/*
Redis port
*/}}
{{- define "hakanai.redisPort" -}}
{{- if .Values.redis.enabled }}
{{- default 6379 .Values.redis.master.service.port }}
{{- else }}
{{- default 6379 .Values.externalRedis.port }}
{{- end }}
{{- end }}

{{/*
Get the Redis password secret name
*/}}
{{- define "hakanai.redisSecretName" -}}
{{- if .Values.redis.enabled }}
{{- if .Values.redis.auth.existingSecret }}
{{- .Values.redis.auth.existingSecret }}
{{- else }}
{{- printf "%s-redis" .Release.Name }}
{{- end }}
{{- else if .Values.externalRedis.existingSecret }}
{{- .Values.externalRedis.existingSecret }}
{{- else }}
{{- printf "%s-redis-external" (include "hakanai.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Get the Redis password secret key
*/}}
{{- define "hakanai.redisSecretPasswordKey" -}}
{{- if .Values.redis.enabled }}
{{- if .Values.redis.auth.existingSecretPasswordKey }}
{{- .Values.redis.auth.existingSecretPasswordKey }}
{{- else -}}
redis-password
{{- end }}
{{- else if .Values.externalRedis.existingSecretPasswordKey }}
{{- .Values.externalRedis.existingSecretPasswordKey }}
{{- else -}}
redis-password
{{- end }}
{{- end }}