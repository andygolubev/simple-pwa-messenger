resource "google_firestore_database" "default" {
  project     = var.project_id
  name        = var.database_name
  location_id = var.location_id
  type        = "FIRESTORE_NATIVE"
}

resource "google_firestore_index" "messages_created_at" {
  project     = var.project_id
  database    = google_firestore_database.default.name
  collection  = "messages"
  query_scope = "COLLECTION"

  fields {
    field_path = "createdAt"
    order      = "ASCENDING"
  }
}

resource "google_firestore_index" "rooms_polling" {
  project     = var.project_id
  database    = google_firestore_database.default.name
  collection  = "rooms"
  query_scope = "COLLECTION"

  fields {
    field_path   = "participants"
    array_config = "CONTAINS"
  }

  fields {
    field_path = "lastMessageAt"
    order      = "DESCENDING"
  }
}
