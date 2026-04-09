variable "project_id" {
  description = "Google Cloud project ID where services will be enabled."
  type        = string
}

variable "services" {
  description = "List of Google APIs to enable for the project."
  type        = list(string)
}
