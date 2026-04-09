locals {
  resolved_secret_names = {
    jwt_signing_key   = lookup(var.secret_names, "jwt_signing_key", "jwt-signing-key")
    vapid_private_key = lookup(var.secret_names, "vapid_private_key", "vapid-private-key")
    vapid_public_key  = lookup(var.secret_names, "vapid_public_key", "vapid-public-key")
  }

  # for_each cannot consume a sensitive map; use nonsensitive() only to decide which keys get a version.
  initial_secret_keys_with_values = toset([
    for key, value in nonsensitive(var.secret_initial_values) : key
    if contains(keys(local.resolved_secret_names), key) && trimspace(value) != ""
  ])
}

resource "google_secret_manager_secret" "secrets" {
  for_each = local.resolved_secret_names

  project   = var.project_id
  secret_id = each.value

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "initial" {
  for_each = local.initial_secret_keys_with_values

  secret      = google_secret_manager_secret.secrets[each.key].id
  secret_data = var.secret_initial_values[each.key]
}

resource "google_secret_manager_secret_iam_member" "function_accessor" {
  for_each = google_secret_manager_secret.secrets

  project   = var.project_id
  secret_id = each.value.secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${var.function_service_account_email}"
}
