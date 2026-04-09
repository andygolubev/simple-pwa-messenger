variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "region" {
  description = "Primary region for regional resources."
  type        = string
}

variable "environment" {
  description = "Deployment environment name, such as dev or prod."
  type        = string
}

variable "firestore_location" {
  description = "Firestore location (region or multi-region)."
  type        = string
}

variable "function_name" {
  description = "Cloud Functions v2 function name."
  type        = string
  default     = "messenger-api"
}

variable "function_runtime" {
  description = "Cloud Functions runtime."
  type        = string
  default     = "nodejs20"
}

variable "function_entry_point" {
  description = "Cloud Functions entry point symbol."
  type        = string
  default     = "handler"
}

variable "function_source_archive" {
  description = "Path to zipped function source code artifact."
  type        = string
  default     = "../functions/dist/function.zip"
}

variable "function_memory_mb" {
  description = "Function memory size in MB."
  type        = number
  default     = 256
}

variable "function_timeout_seconds" {
  description = "Function timeout in seconds."
  type        = number
  default     = 60
}

variable "function_min_instances" {
  description = "Minimum number of function instances."
  type        = number
  default     = 0
}

variable "function_max_instances" {
  description = "Maximum number of function instances."
  type        = number
  default     = 5
}

variable "function_environment_variables" {
  description = "Environment variables injected into Cloud Function."
  type        = map(string)
  default     = {}
}

variable "secret_version_destroy_ttl" {
  description = "TTL for delayed secret version destruction (for example 0s, 86400s)."
  type        = string
  default     = "0s"
}

variable "identity_platform_autodelete_anonymous_users" {
  description = "Automatically delete anonymous users in Identity Platform."
  type        = bool
  default     = true
}

variable "google_oauth_client_id" {
  description = "Optional custom OAuth client ID for Google IdP."
  type        = string
  default     = ""
}

variable "google_oauth_client_secret" {
  description = "Optional custom OAuth client secret for Google IdP."
  type        = string
  default     = ""
  sensitive   = true
}

variable "secret_names" {
  description = "Secret Manager secret IDs used by the platform."
  type        = map(string)
  default = {
    jwt_signing_key   = "jwt-signing-key"
    vapid_private_key = "vapid-private-key"
    vapid_public_key  = "vapid-public-key"
  }
}

variable "secret_initial_values" {
  description = "Optional initial secret values keyed by logical name."
  type        = map(string)
  default     = {}
  sensitive   = true
}
