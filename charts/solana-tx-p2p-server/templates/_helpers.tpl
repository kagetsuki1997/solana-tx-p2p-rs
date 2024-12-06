{{/*
Expand the name of the chart.
*/}}
{{- define "solana-tx-p2p-server.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "solana-tx-p2p-server.fullname" -}}
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
{{- define "solana-tx-p2p-server.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "solana-tx-p2p-server.labels" -}}
helm.sh/chart: {{ include "solana-tx-p2p-server.chart" . }}
{{ include "solana-tx-p2p-server.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: solana-tx-p2p-server
{{- end }}

{{/*
Selector labels
*/}}
{{- define "solana-tx-p2p-server.selectorLabels" -}}
app.kubernetes.io/name: {{ include "solana-tx-p2p-server.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: server
{{- end }}

{{- define "solana-tx-p2p-server.container.name" -}}
solana-tx-p2p-server
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "solana-tx-p2p-server.serviceAccount.name" -}}
{{- default (include "solana-tx-p2p-server.fullname" .) .Values.serviceAccount.name }}
{{- end }}

{{- define "solana-tx-p2p-server.certificate.secretName" -}}
{{- printf "%s-certificate" (include "solana-tx-p2p-server.fullname" .) }}
{{- end }}

{{- define "solana-tx-p2p-server.host" -}}
{{- printf "%s.%s" (include "solana-tx-p2p-server.fullname" .) (include "common.internalDomain" .) }}
{{- end }}

{{- define "solana-tx-p2p-server.httpEndpoint" -}}
{{- printf "http://%s:%s" (include "solana-tx-p2p-server.host" .) (include "common.httpPort" .) }}
{{- end }}

{{- define "solana-tx-p2p-server.httpsEndpoint" -}}
{{- printf "https://%s:%s" (include "solana-tx-p2p-server.host" .) (include "common.httpsPort" .) }}
{{- end }}

{{- define "solana-tx-p2p-server.grpcEndpoint" -}}
{{- printf "http://%s:%s" (include "solana-tx-p2p-server.host" .) (include "common.grpcPort" .) }}
{{- end }}

{{- define "solana-tx-p2p-server.metricsEndpoint" -}}
{{- printf "http://%s:%s" (include "solana-tx-p2p-server.host" .) (include "common.metricsPort" .) }}
{{- end }}
