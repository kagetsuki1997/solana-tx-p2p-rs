{{- define "common.httpPort" -}}
80
{{- end }}

{{- define "common.httpsPort" -}}
443
{{- end }}

{{- define "common.grpcPort" -}}
50051
{{- end }}

{{- define "common.metricsPort" -}}
9090
{{- end }}

{{- define "common.internalDomain" -}}
{{- printf "%s.svc.%s" .Release.Namespace .Values.clusterDomain }}
{{- end }}

{{- define "common.podInfoEnvs" -}}
- name: POD_NAMESPACE
  valueFrom:
    fieldRef:
      apiVersion: v1
      fieldPath: metadata.namespace
- name: POD_NAME
  valueFrom:
    fieldRef:
      fieldPath: metadata.name
- name: POD_IP
  valueFrom:
    fieldRef:
      apiVersion: v1
      fieldPath: status.podIP
- name: NODE_NAME
  valueFrom:
    fieldRef:
      apiVersion: v1
      fieldPath: spec.nodeName
- name: ISTIO_REV
  valueFrom:
    fieldRef:
      apiVersion: v1
      fieldPath: metadata.annotations['istio.io/rev']
{{- end }}
