variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "autodelete_anonymous_users" {
  description = "Automatically remove anonymous users from Identity Platform."
  type        = bool
  default     = true
}

variable "google_oauth_client_id" {
  description = "Optional OAuth client ID for Google IdP. If empty, Google-managed defaults apply."
  type        = string
  default     = ""
}

variable "google_oauth_client_secret" {
  description = "Optional OAuth client secret for Google IdP."
  type        = string
  default     = ""
  sensitive   = true
}
