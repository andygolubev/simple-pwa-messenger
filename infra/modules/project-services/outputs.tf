output "enabled_services" {
  description = "Set of enabled project services."
  value       = keys(google_project_service.required)
}
