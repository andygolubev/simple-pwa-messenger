project_id              = "REPLACE_WITH_PROD_PROJECT_ID"
region                  = "europe-west1"
environment             = "prod"
firestore_location      = "europe-west1"
function_name           = "messenger-api"
function_runtime        = "nodejs20"
function_entry_point    = "handler"
function_source_archive = "../functions/dist/function.zip"

function_environment_variables = {
  NODE_ENV = "production"
}

# Optional custom OAuth settings for Identity Platform (leave blank for defaults)
google_oauth_client_id     = ""
google_oauth_client_secret = ""

# Optional secret bootstrap values. Prefer loading real values through secure pipeline secrets.
secret_initial_values = {
  jwt_signing_key   = ""
  vapid_private_key = ""
  vapid_public_key  = ""
}
