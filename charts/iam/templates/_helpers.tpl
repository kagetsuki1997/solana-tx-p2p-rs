{{/*
Expand the name of the chart.
*/}}
{{- define "zta-iam.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "zta-iam.fullname" -}}
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
{{- define "zta-iam.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "zta-iam.labels" -}}
helm.sh/chart: {{ include "zta-iam.chart" . }}
{{ include "zta-iam.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: zta-iam
{{- end }}

{{/*
Selector labels
*/}}
{{- define "zta-iam.selectorLabels" -}}
app.kubernetes.io/name: {{ include "zta-iam.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: server
{{- end }}

{{- define "zta-iam.container.name" -}}
zta-iam
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "zta-iam.serviceAccount.name" -}}
{{- default (include "zta-iam.fullname" .) .Values.serviceAccount.name }}
{{- end }}

{{- define "zta-iam.certificate.secretName" -}}
{{- printf "%s-certificate" (include "zta-iam.fullname" .) }}
{{- end }}

{{- define "zta-iam.host" -}}
{{- printf "%s.%s" (include "zta-iam.fullname" .) (include "common.internalDomain" .) }}
{{- end }}

{{- define "zta-iam.httpEndpoint" -}}
{{- printf "http://%s:%s" (include "zta-iam.host" .) (include "common.httpPort" .) }}
{{- end }}

{{- define "zta-iam.httpsEndpoint" -}}
{{- printf "https://%s:%s" (include "zta-iam.host" .) (include "common.httpsPort" .) }}
{{- end }}

{{- define "zta-iam.grpcEndpoint" -}}
{{- printf "http://%s:%s" (include "zta-iam.host" .) (include "common.grpcPort" .) }}
{{- end }}

{{- define "zta-iam.metricsEndpoint" -}}
{{- printf "http://%s:%s" (include "zta-iam.host" .) (include "common.metricsPort" .) }}
{{- end }}
