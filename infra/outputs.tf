output "project_id" {
  description = "Target Google Cloud project ID."
  value       = var.project_id
}

output "function_service_account_email" {
  description = "Service account email used by Cloud Functions."
  value       = module.iam.function_service_account_email
}

output "function_name" {
  description = "Deployed Cloud Functions v2 resource name."
  value       = module.cloud_functions.function_name
}

output "function_uri" {
  description = "Public HTTPS URI for the API function."
  value       = module.cloud_functions.function_uri
}

output "secret_ids" {
  description = "Created Secret Manager secret resource names keyed by logical identifier."
  value       = module.secret_manager.secret_ids
}

output "firestore_database_name" {
  description = "Firestore database resource name."
  value       = module.firestore.database_name
}
