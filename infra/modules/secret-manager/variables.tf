variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "function_service_account_email" {
  description = "Cloud Functions service account granted access to secrets."
  type        = string
}

variable "secret_names" {
  description = "Secret IDs keyed by logical secret name."
  type        = map(string)
}

variable "secret_initial_values" {
  description = "Optional initial secret payloads keyed by logical secret name."
  type        = map(string)
  default     = {}
  sensitive   = true
}

variable "secret_version_destroy_ttl" {
  description = "TTL for delayed secret version destruction (for example 0s, 86400s)."
  type        = string
  default     = "0s"
}
