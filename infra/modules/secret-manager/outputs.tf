output "secret_ids" {
  description = "Secret resource IDs keyed by logical names."
  value = {
    for key, secret in google_secret_manager_secret.secrets :
    key => secret.id
  }
}

output "secret_version_ids" {
  description = "Initial secret version IDs where initial values were provided."
  value = {
    for key, version in google_secret_manager_secret_version.initial :
    key => version.id
  }
  sensitive = true
}
