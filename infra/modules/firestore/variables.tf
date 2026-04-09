variable "project_id" {
  description = "Google Cloud project ID."
  type        = string
}

variable "location_id" {
  description = "Firestore database location."
  type        = string
}

variable "database_name" {
  description = "Firestore database ID."
  type        = string
  default     = "(default)"
}
