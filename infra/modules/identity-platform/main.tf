resource "google_identity_platform_config" "default" {
  provider = google-beta
  project  = var.project_id

  autodelete_anonymous_users = var.autodelete_anonymous_users
}

resource "google_identity_platform_default_supported_idp_config" "google" {
  count = trimspace(var.google_oauth_client_id) != "" && trimspace(var.google_oauth_client_secret) != "" ? 1 : 0

  provider = google-beta
  project  = var.project_id

  idp_id        = "google.com"
  enabled       = true
  client_id     = trimspace(var.google_oauth_client_id)
  client_secret = trimspace(var.google_oauth_client_secret)

  depends_on = [google_identity_platform_config.default]
}
