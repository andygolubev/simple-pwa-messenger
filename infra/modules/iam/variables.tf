variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "environment" {
  description = "Environment suffix for generated IAM resources."
  type        = string
}

variable "function_service_account_id" {
  description = "Account ID (not email) for the Cloud Functions service account."
  type        = string
}
