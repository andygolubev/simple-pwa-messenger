variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "region" {
  description = "Google Cloud region for function deployment."
  type        = string
}

variable "environment" {
  description = "Environment name used for naming resources."
  type        = string
}

variable "function_name" {
  description = "Cloud Functions v2 function name."
  type        = string
}

variable "runtime" {
  description = "Cloud Functions runtime."
  type        = string
}

variable "entry_point" {
  description = "Cloud Functions entry point symbol."
  type        = string
}

variable "source_archive" {
  description = "Path to source archive zip uploaded as function source."
  type        = string
}

variable "service_account_email" {
  description = "Service account email used by the function."
  type        = string
}

variable "memory_mb" {
  description = "Memory allocation for function in MB."
  type        = number
  default     = 256
}

variable "timeout_seconds" {
  description = "Function timeout in seconds."
  type        = number
  default     = 60
}

variable "min_instance_count" {
  description = "Minimum function instance count."
  type        = number
  default     = 0
}

variable "max_instance_count" {
  description = "Maximum function instance count."
  type        = number
  default     = 5
}

variable "environment_variables" {
  description = "Runtime environment variables for the function."
  type        = map(string)
  default     = {}
}

variable "allow_unauthenticated_invoker" {
  description = "Grant allUsers run.invoker on the generated Cloud Run service."
  type        = bool
  default     = true
}
