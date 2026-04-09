output "function_service_account_email" {
  description = "Email of the Cloud Functions runtime service account."
  value       = google_service_account.functions.email
}

output "function_service_account_name" {
  description = "Fully-qualified name of the Cloud Functions runtime service account."
  value       = google_service_account.functions.name
}
