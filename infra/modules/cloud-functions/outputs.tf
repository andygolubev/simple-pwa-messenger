output "function_name" {
  description = "Cloud Function name."
  value       = google_cloudfunctions2_function.api.name
}

output "function_uri" {
  description = "Public URI for the deployed Cloud Function."
  value       = google_cloudfunctions2_function.api.service_config[0].uri
}

output "source_bucket_name" {
  description = "Name of the bucket storing function source archives."
  value       = google_storage_bucket.function_source.name
}
