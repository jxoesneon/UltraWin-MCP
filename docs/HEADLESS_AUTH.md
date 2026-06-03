Test content
# Headless Authentication for UltraWin

This document describes the procedure for setting up headless authentication using Google Service Accounts with Domain-Wide Delegation. This configuration is essential for enterprise automation where interactive browser logins are not possible.

## Google Service Account Setup

1. Create a Service Account: Go to the Google Cloud Console, navigate to IAM & Admin > Service Accounts, and create a new service account.
2. Generate a JSON Key: Create a new JSON key for the service account and download it.
3. Domain-Wide Delegation:
   - In the Cloud Console, enable Domain-Wide Delegation for the service account.
   - Note the Unique ID (Client ID) of the service account.
   - In the Google Workspace Admin Console, go to Security > Access and data control > API controls > Manage Domain Wide Delegation.
   - Add a new API client with the service account's Client ID and the required OAuth scopes.

## Configuration

Store the path to your service account JSON key in your .env file:

\\\env
GOOGLE_APPLICATION_CREDENTIALS="C:\path\to\your\service-account-key.json"
\\\`n
## Benefits

- No Interactive Login: Bypasses the requirement for an interactive browser-based OAuth flow.
- Service-to-Service: Enables secure communication between UltraWin and Google APIs.
- Impersonation: With Domain-Wide Delegation, the service account can impersonate a user in your workspace.
