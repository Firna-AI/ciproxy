{{/* One public selector binds the CDN key name to a numbered version. */}}
{{/* Missing/invalid selectors select nonexistent version 0 at mount time. */}}
{{- define "ciproxy.cdnSecretVersion" -}}
{{- $selector := default "" .Values.controlPlane.cdnSigningKeyName -}}
{{- $prefix := printf "%s-images-v" .Values.prefix -}}
{{- $version := int64 0 -}}
{{- if hasPrefix $prefix $selector -}}
{{- $suffix := trimPrefix $prefix $selector -}}
{{- if regexMatch "^[0-9]+$" $suffix -}}
{{- $number := regexReplaceAll "^0+" $suffix "" | int64 -}}
{{- if gt $number (int64 0) -}}
{{- $version = $number -}}
{{- end -}}
{{- end -}}
{{- end -}}
{{- $version -}}
{{- end -}}

{{- define "ciproxy.cdnSigningKeyName" -}}
{{- printf "%s-images-v%s" .Values.prefix (include "ciproxy.cdnSecretVersion" .) -}}
{{- end -}}

{{/* Hash the full provider spec so no file mapping changes under old pods. */}}
{{- define "ciproxy.secretProviderSpec" -}}
{{- $version := include "ciproxy.cdnSecretVersion" . -}}
{{- $files := list -}}
{{- range $file, $resource := .Values.controlPlane.secrets -}}
{{- $selectedVersion := ternary $version "latest" (eq $file "cdn-signing-key") -}}
{{- $files = append $files (dict "resourceName" (printf "%s/versions/%s" $resource $selectedVersion) "path" $file) -}}
{{- end -}}
{{- dict "provider" "gke" "parameters" (dict "secrets" (toYaml $files)) | toYaml -}}
{{- end -}}

{{- define "ciproxy.secretProviderName" -}}
{{- printf "control-plane-%s" (include "ciproxy.secretProviderSpec" . | sha256sum | trunc 48) -}}
{{- end -}}
