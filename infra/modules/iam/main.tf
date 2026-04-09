locals {
  project_roles = [
    "roles/datastore.user",
    "roles/logging.logWriter",
    "roles/secretmanager.secretAccessor",
  ]
}

resource "google_service_account" "functions" {
  account_id   = var.function_service_account_id
  project      = var.project_id
  display_name = "Messenger Functions service account (${var.environment})"
}

resource "google_project_iam_member" "function_project_roles" {
  for_each = toset(local.project_roles)

  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.functions.email}"
}
