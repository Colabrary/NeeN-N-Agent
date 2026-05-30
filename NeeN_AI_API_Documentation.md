# NeeN AI Complete API Documentation

**For Desktop Overlay Autonomous Applications**

## Table of Contents

1. [Overview](#overview)
2. [Base URLs](#base-urls)
3. [Authentication Methods](#authentication-methods)
4. [Core API Categories](#core-api-categories)
5. [Authentication APIs](#authentication-apis)
6. [NeeN AI Chat APIs](#neen-ai-chat-apis)
7. [Text-to-Speech (TTS) APIs](#text-to-speech-tts-apis)
8. [Voice Agent APIs](#voice-agent-apis)
9. [Lead Capture APIs](#lead-capture-apis)
10. [WhatsApp Integration APIs](#whatsapp-integration-apis)
11. [Supported AI Actions](#supported-ai-actions)
12. [Multi-Language Support](#multi-language-support)
13. [Available TTS Voices](#available-tts-voices)
14. [Error Handling](#error-handling)
15. [Rate Limits](#rate-limits)
16. [Desktop Overlay Integration Tips](#desktop-overlay-integration-tips)
17. [Code Examples](#code-examples)

---

## Overview

NeeN AI is a comprehensive CRM platform with AI-powered voice and chat capabilities. This documentation covers all APIs needed to build autonomous desktop overlay applications that can interact with the system through natural language, voice commands, and automated actions.

**Key Features:**
- Natural language AI chat with CRM actions
- Multi-language support (21+ languages)
- Text-to-Speech with 116+ voices across 40+ languages
- Voice agent with speech-to-text processing
- Lead capture from multiple sources
- WhatsApp integration
- Real-time conversation management

---

## Base URLs

```
Production:  https://api.9ance.com/api/
Development: http://localhost:8000/api/
```

---

## Authentication Methods

### 1. JWT Bearer Token (Main Authentication)
```
Authorization: Bearer <access_token>
```
Used for: User-specific operations, CRM data access, authenticated AI chat

### 2. AI Access Key (Public/Widget Access)
```
X-AI-Access-Key: <your-access-key>
```
Used for: Public AI chat, TTS without user login, voice agent

### 3. Lead Capture API Key
```
x-api-key: lc_live_xxxxxxxxxxxxx
```
Used for: Lead capture from external sources, website forms

---

## Core API Categories

### 1. Authentication APIs
- User signup and login
- Token management and refresh
- Password reset functionality

### 2. NeeN AI Chat APIs
- Natural language conversation
- CRM action execution
- Conversation management
- Multi-model AI support

### 3. Text-to-Speech (TTS) APIs
- Speech synthesis in 40+ languages
- 116+ voice options
- Public and authenticated access

### 4. Voice Agent APIs
- Speech-to-text processing
- AI response generation
- Audio response synthesis
- Cloud telephony integration

### 5. Lead Capture APIs
- Public lead capture
- Multi-platform integration
- API key management
- Webhook support

### 6. WhatsApp Integration APIs
- Multi-account management
- QR code authentication
- Message sending/receiving
- Auto-reply with AI

---

## Authentication APIs

### User Signup
Create a new user account with 14-day trial.

**Endpoint:** `POST /auth/signup/`

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123",
  "name": "John Doe",
  "company_name": "My Company",
  "mobile_number": "9876543210",
  "employee_range_choice": "10-50"
}
```

**Required Fields:**
- `email` (string) - Valid email address
- `password` (string) - Minimum 6 characters

**Success Response (201):**
```json
{
  "token": "jwt_token_here",
  "user": {
    "id": "user-uuid",
    "email": "user@example.com",
    "name": "John Doe"
  },
  "tenant": {
    "company_name": "My Company",
    "is_admin": true,
    "role": "Administrator"
  },
  "subscription": {
    "plan_name": "Startup Plus Trial",
    "status": "trial",
    "trial_days_remaining": 14
  }
}
```

### User Login
Authenticate user and get JWT tokens.

**Endpoint:** `POST /auth/login/`

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123",
  "platform": "mobile"
}
```

**Success Response (200):**
```json
{
  "refresh": "refresh_token_here",
  "access": "access_token_here",
  "user_id": "user-uuid",
  "permissions": {
    "leads": ["create", "view", "update", "delete"],
    "contacts": ["create", "view", "update", "delete"]
  },
  "user": {
    "id": "user-uuid",
    "email": "user@example.com",
    "name": "John Doe"
  }
}
```

### Token Refresh
Refresh expired access token.

**Endpoint:** `POST /auth/token/refresh/`

**Request:**
```json
{
  "refresh": "refresh_token_here"
}
```

**Response:**
```json
{
  "access": "new_access_token_here"
}
```

### Get Current User
Get authenticated user's profile.

**Endpoint:** `GET /auth/me/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "id": "user-uuid",
  "email": "user@example.com",
  "name": "John Doe",
  "tenant": {
    "id": "tenant-uuid",
    "name": "My Company"
  },
  "role": {
    "name": "Administrator",
    "permissions": {}
  }
}
```
---

## NeeN AI Chat APIs

### Main Chat Endpoint
Send messages to NeeN AI and get responses with CRM actions.

**Endpoint:** `POST /ai/chat/`
**Headers:** `Authorization: Bearer <access_token>`

**Request:**
```json
{
  "message": "Add a new lead named Rahul with phone 9876543210",
  "conversation_id": "optional-conversation-uuid",
  "model": "openai/gpt-4o-mini"
}
```

**Parameters:**
- `message` (string, required) - User's message/command
- `conversation_id` (string, optional) - Existing conversation ID
- `model` (string, optional) - AI model to use

**Success Response (200):**
```json
{
  "conversation_id": "conversation-uuid",
  "message": "Done! I've added Rahul as a new lead with phone 9876543210 👍",
  "action": {
    "type": "create_lead",
    "data": {
      "first_name": "Rahul",
      "phone": "9876543210"
    }
  },
  "action_result": {
    "success": true,
    "entity_type": "lead",
    "entity_id": "lead-uuid",
    "data": {
      "id": "lead-uuid",
      "first_name": "Rahul",
      "phone": "9876543210",
      "created_at": "2026-01-29T10:00:00Z"
    }
  }
}
```

### List Conversations
Get all user's AI conversations.

**Endpoint:** `GET /ai/conversations/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "conversations": [
    {
      "id": "conversation-uuid",
      "title": "Lead management discussion",
      "model": "openai/gpt-4o-mini",
      "created_at": "2026-01-29T10:00:00Z",
      "updated_at": "2026-01-29T10:30:00Z"
    }
  ]
}
```

### Get Conversation Details
Get specific conversation with all messages.

**Endpoint:** `GET /ai/conversations/{conversation_id}/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "id": "conversation-uuid",
  "title": "Lead management discussion",
  "model": "openai/gpt-4o-mini",
  "messages": [
    {
      "id": "msg_001",
      "role": "user",
      "content": "Show me today's leads",
      "timestamp": "2026-01-29T10:00:00Z"
    },
    {
      "id": "msg_002",
      "role": "assistant",
      "content": "Here are today's leads:",
      "timestamp": "2026-01-29T10:00:05Z",
      "action": {
        "type": "list_leads",
        "result": {"success": true, "count": 5}
      }
    }
  ]
}
```

### Available AI Models
List all available AI models.

**Endpoint:** `GET /ai/models/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "models": [
    {
      "id": "openai/gpt-4o-mini",
      "name": "GPT-4o Mini",
      "provider": "OpenAI",
      "is_free": false,
      "is_default": true
    },
    {
      "id": "anthropic/claude-3-haiku",
      "name": "Claude 3 Haiku",
      "provider": "Anthropic",
      "is_free": false,
      "is_default": false
    }
  ]
}
```

### Public AI Chat (No User Login)
For embedding AI chat without user authentication.

**Endpoint:** `POST /ai/public/chat/`
**Headers:** `X-AI-Access-Key: <your-access-key>`

**Request:**
```json
{
  "message": "Hello, I need help with CRM",
  "session_id": "optional-session-uuid"
}
```

**Response:**
```json
{
  "session_id": "session-uuid",
  "message": "Hi! I'm NeeN, your AI CRM assistant. How can I help you today?",
  "action": {"type": "none"},
  "action_result": null
}
```
---

## Text-to-Speech (TTS) APIs

### Synthesize Speech
Convert text to speech audio.

**Endpoint:** `POST /tts/synthesize/`
**Headers:** `Authorization: Bearer <access_token>`

**Request:**
```json
{
  "text": "Hello, how can I help you today?",
  "voice": "en_US-lessac-medium",
  "speed": 1.0
}
```

**Parameters:**
- `text` (string, required) - Text to synthesize (max 5000 chars)
- `voice` (string, optional) - Voice model ID (default: en_US-lessac-medium)
- `speed` (float, optional) - Speech speed 0.5-2.0 (default: 1.0)

**Response:** Binary WAV audio file (`audio/wav`)

### List Available Voices
Get all available voice models.

**Endpoint:** `GET /tts/voices/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "voices": [
    {
      "id": "en_US-lessac-medium",
      "name": "Lessac (US English)",
      "language": "en-US",
      "quality": "medium",
      "gender": "female",
      "downloaded": true,
      "is_default": true
    },
    {
      "id": "hi_IN-swara-medium",
      "name": "Swara (Hindi)",
      "language": "hi-IN",
      "quality": "medium",
      "gender": "female",
      "downloaded": false,
      "is_default": false
    }
  ],
  "default_voice": "en_US-lessac-medium"
}
```

### List Voices by Language
Get voices for specific language.

**Endpoint:** `GET /tts/voices/language/{language_code}/`
**Example:** `/tts/voices/language/en-US/`

**Response:**
```json
{
  "language": "en-US",
  "voices": [
    {
      "id": "en_US-lessac-medium",
      "name": "Lessac",
      "language": "en-US",
      "quality": "medium",
      "gender": "female",
      "downloaded": true,
      "is_default": true
    }
  ],
  "total": 15
}
```

### List Supported Languages
Get all languages with available voices.

**Endpoint:** `GET /tts/languages/`

**Response:**
```json
{
  "languages": [
    {"code": "en-US", "name": "English (US)"},
    {"code": "en-GB", "name": "English (UK)"},
    {"code": "hi-IN", "name": "Hindi"},
    {"code": "de-DE", "name": "German"},
    {"code": "fr-FR", "name": "French"},
    {"code": "es-ES", "name": "Spanish (Spain)"},
    {"code": "zh-CN", "name": "Chinese (Mandarin)"},
    {"code": "ja-JP", "name": "Japanese"}
  ],
  "total": 40
}
```

### Public TTS (No User Login)
For AI widgets without user authentication.

**Endpoint:** `POST /tts/public/synthesize/`
**Headers:** `X-AI-Access-Key: <your-access-key>`

**Request:**
```json
{
  "text": "Hello, how can I help you?",
  "voice": "en_US-lessac-medium",
  "speed": 1.0
}
```

**Note:** Maximum 2000 characters for public API.

### TTS Health Check
Check if TTS service is available.

**Endpoint:** `GET /tts/health/`

**Response:**
```json
{
  "status": "healthy",
  "piper_installed": true,
  "default_voice_available": true,
  "default_voice": "en_US-lessac-medium",
  "models_directory": "/app/piper_models"
}
```
---

## Voice Agent APIs

### Process Voice Input
Process audio input and return audio response (Speech-to-Text + AI + Text-to-Speech).

**Endpoint:** `POST /voice-agent/process/`
**Headers:** `X-AI-Access-Key: <your-access-key>`

**Request (Audio Input):**
```json
{
  "audio": "<base64_encoded_audio>",
  "encoding": "LINEAR16",
  "sample_rate": 8000,
  "language": "en-US",
  "voice": "en_US-lessac-medium",
  "session_id": "optional-session-uuid",
  "speed": 1.0
}
```

**Request (Text Input - for testing):**
```json
{
  "text": "Hello, how are you?",
  "voice": "en_US-lessac-medium",
  "session_id": "optional-session-uuid"
}
```

**Parameters:**
- `audio` (string) - Base64 encoded audio data
- `text` (string) - Text input (alternative to audio)
- `encoding` (string) - Audio encoding (LINEAR16, FLAC, MULAW, AMR, etc.)
- `sample_rate` (int) - Sample rate in Hz (8000, 16000, etc.)
- `language` (string) - Language code for STT (default: en-US)
- `voice` (string) - TTS voice ID (default: en_US-lessac-medium)
- `session_id` (string) - Session ID for conversation continuity
- `speed` (float) - Speech speed 0.5-2.0 (default: 1.0)
- `return_json` (bool) - Return JSON instead of audio (default: false)

**Response (Default - Audio):**
- Content-Type: `audio/wav`
- Binary WAV audio data

**Response (with return_json=true):**
```json
{
  "success": true,
  "transcript": "user said this",
  "stt_confidence": 0.95,
  "response_text": "AI response text",
  "audio_base64": "<base64_encoded_audio>",
  "session_id": "session-uuid",
  "voice": "en_US-lessac-medium",
  "language": "en-US"
}
```

### Speech-to-Text Only
Transcribe audio without AI processing.

**Endpoint:** `POST /voice-agent/stt/`
**Headers:** `X-AI-Access-Key: <your-access-key>`

**Request:**
```json
{
  "audio": "<base64_encoded_audio>",
  "encoding": "LINEAR16",
  "sample_rate": 8000,
  "language": "en-US"
}
```

**Response:**
```json
{
  "success": true,
  "transcript": "transcribed text",
  "confidence": 0.95,
  "language": "en-US"
}
```

### Voice Agent Health Check
Check voice agent service availability.

**Endpoint:** `GET /voice-agent/health/`

**Response:**
```json
{
  "status": "healthy",
  "stt_available": true,
  "tts_available": true,
  "default_voice": "en_US-lessac-medium",
  "supported_encodings": ["LINEAR16", "FLAC", "MULAW", "AMR", "AMR_WB", "OGG_OPUS"],
  "supported_sample_rates": [8000, 16000, 22050, 44100, 48000]
}
```
---

## Lead Capture APIs

### Public Lead Capture (Simple)
Capture leads without API key authentication.

**Endpoint:** `POST /public-lead/create/`

**Request:**
```json
{
  "name": "Jane Smith",
  "email": "jane@example.com",
  "phone": "+919876543210",
  "company": "Tech Solutions",
  "source": "Website Contact Form",
  "notes": "Interested in premium features"
}
```

**Required Fields:**
- `name` (string) - Full name
- At least one of: `email` or `phone`

**Success Response (201):**
```json
{
  "success": true,
  "message": "Lead created successfully",
  "lead_id": "lead-uuid",
  "data": {
    "id": "lead-uuid",
    "name": "Jane Smith",
    "email": "jane@example.com",
    "phone": "+919876543210",
    "company": "Tech Solutions",
    "source": "Website Contact Form",
    "status": "New"
  }
}
```

### Universal Lead Capture (API Key Required)
Advanced lead capture with API key authentication.

**Endpoint:** `POST /v1/leads/capture/`
**Headers:** `x-api-key: lc_live_xxxxxxxxxxxxx`

**Request:**
```json
{
  "source": "facebook",
  "campaign": "summer_2026",
  "first_name": "John",
  "last_name": "Doe",
  "email": "john@example.com",
  "phone": "+919876543210",
  "company": "Acme Corp",
  "notes": "Interested in premium",
  "custom_fields": {
    "budget": "50000",
    "industry": "Technology"
  }
}
```

**Required Fields:**
- `first_name` (string) - First name
- At least one of: `email` or `phone`

**Success Response (201):**
```json
{
  "success": true,
  "message": "Lead captured successfully",
  "lead_id": "lead-uuid",
  "data": {
    "id": "lead-uuid",
    "first_name": "John",
    "last_name": "Doe",
    "email": "john@example.com",
    "phone": "+919876543210",
    "company": "Acme Corp",
    "source": "facebook",
    "status": "New"
  }
}
```
---

## WhatsApp Integration APIs

### List WhatsApp Accounts
Get all WhatsApp accounts for current user.

**Endpoint:** `GET /webwhatsapp/accounts/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "count": 2,
  "accounts": [
    {
      "id": "account-uuid",
      "account_name": "Sales WhatsApp",
      "phone_number": "+919876543210",
      "status": "connected",
      "status_message": "Connected",
      "is_default": true,
      "is_active": true,
      "auto_reply_enabled": true,
      "daily_limit": 500,
      "messages_sent_today": 45,
      "last_connected_at": "2026-01-29T10:00:00Z"
    }
  ]
}
```

### Get QR Code for Authentication
Scan QR code with WhatsApp mobile app to connect.

**Endpoint:** `GET /webwhatsapp/accounts/{account_id}/qr/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "qr_code": "data:image/png;base64,iVBORw0KGgo...",
  "session_id": "session-uuid",
  "expires_in": 60,
  "message": "Scan QR code with WhatsApp"
}
```

### Check Connection Status
Check WhatsApp connection status.

**Endpoint:** `GET /webwhatsapp/accounts/{account_id}/status/`
**Headers:** `Authorization: Bearer <access_token>`

**Response:**
```json
{
  "status": "connected",
  "phone_number": "+919876543210",
  "message": "WhatsApp connected",
  "last_seen": "2026-01-29T10:30:00Z"
}
```

**Status Values:**
- `disconnected` - Not connected, needs QR scan
- `qr_pending` - QR code generated, waiting for scan
- `connecting` - Connection in progress
- `connected` - Successfully connected
- `error` - Connection error

### Send WhatsApp Message
Send message via WhatsApp.

**Endpoint:** `POST /webwhatsapp/send/`
**Headers:** `Authorization: Bearer <access_token>`

**Request:**
```json
{
  "to_phone": "+919876543210",
  "message": "Hello! This is a test message.",
  "account_id": "optional-account-uuid",
  "to_name": "John Doe",
  "entity_type": "lead",
  "entity_id": "lead-uuid"
}
```

**Parameters:**
- `to_phone` (string, required) - Recipient phone with country code
- `message` (string, required) - Message text
- `account_id` (uuid, optional) - Specific account to use
- `to_name` (string, optional) - Recipient name for logging
- `entity_type` (string, optional) - lead, contact, or customer
- `entity_id` (uuid, optional) - Link message to CRM entity

**Response:**
```json
{
  "success": true,
  "message": "Message sent",
  "data": {
    "to_phone": "+919876543210",
    "to_name": "John Doe",
    "message_body": "Hello! This is a test message.",
    "sent_by": "user@example.com",
    "sent_at": "2026-01-29T10:30:00Z",
    "message_id": "msg-uuid"
  }
}
```

### Toggle Auto-Reply
Enable/disable NeeN AI auto-reply for incoming messages.

**Endpoint:** `POST /webwhatsapp/accounts/{account_id}/auto-reply/`
**Headers:** `Authorization: Bearer <access_token>`

**Request:**
```json
{
  "enable": true
}
```

**Response:**
```json
{
  "success": true,
  "auto_reply_enabled": true,
  "message": "Auto-reply enabled for Sales WhatsApp"
}
```

### Get Message History
Get WhatsApp message history.

**Endpoint:** `GET /webwhatsapp/messages/`
**Headers:** `Authorization: Bearer <access_token>`

**Query Parameters:**
- `account_id` (uuid) - Filter by account
- `phone` (string) - Filter by phone number
- `direction` (string) - "incoming" or "outgoing"
- `entity_type` (string) - lead, contact, customer
- `limit` (integer) - Max results (default: 50, max: 100)

**Response:**
```json
{
  "count": 25,
  "messages": [
    {
      "id": "msg-uuid",
      "direction": "outgoing",
      "remote_phone": "+919876543210",
      "remote_name": "John Doe",
      "message_body": "Hello!",
      "status": "delivered",
      "entity_type": "lead",
      "entity_id": "lead-uuid",
      "sent_by": "user@example.com",
      "created_at": "2026-01-29T10:30:00Z"
    }
  ]
}
```
---

## Supported AI Actions

NeeN AI can perform these CRM actions through natural language commands:

### Create Actions
- `create_lead` - Add new lead
  - Example: "Add lead Rahul 9876543210 rahul@gmail.com"
- `create_contact` - Add new contact
  - Example: "Create contact Priya from ABC Corp"
- `create_deal` - Create new deal
  - Example: "Create deal worth 50000 for Rahul"
- `create_activity` - Schedule activity (call, meeting, follow-up, task)
  - Example: "Schedule call with Rahul tomorrow 3pm"
- `create_quote` - Generate quote
  - Example: "Create quote for Rahul with 3 products"
- `create_order` - Create order
  - Example: "Create order for customer ABC Corp"

### List/Search Actions
- `list_leads` - List leads with filters
  - Example: "Show me today's leads"
- `list_contacts` - List contacts
  - Example: "Show all contacts from Mumbai"
- `list_customers` - List customers
  - Example: "Show premium customers"
- `list_deals` - List deals
  - Example: "Show deals worth more than 100000"
- `list_activities` - List activities
  - Example: "Show my pending follow-ups"
- `search` - Search across entities
  - Example: "Find Rahul"

### Update Actions
- `update_lead` - Update lead details
  - Example: "Update Rahul's email to new@email.com"
- `update_deal` - Update deal
  - Example: "Mark deal as won"
- `update_activity_status` - Mark activity complete/cancel
  - Example: "Mark call with Rahul as completed"
- `assign_entity` - Assign lead/contact to team member
  - Example: "Assign Rahul to sales team"

### Dashboard Actions
- `get_dashboard` - Get dashboard summary
  - Example: "Show me dashboard"
- `get_notifications` - Get alerts
  - Example: "Show notifications"
- `get_today_schedule` - Today's activities
  - Example: "What's my schedule today?"
- `get_daily_summary` - Daily performance summary
  - Example: "Show today's performance"

---

## Multi-Language Support

NeeN AI supports conversations in 21+ languages:

### Supported Languages
- **English** - Primary language
- **Hindi / Hinglish (हिंदी)** - Native Hindi and English-Hindi mix
- **Gujarati (ગુજરાતી)** - Regional Indian language
- **Marathi (मराठी)** - Regional Indian language
- **Tamil (தமிழ்)** - Regional Indian language
- **Telugu (తెలుగు)** - Regional Indian language
- **Kannada (ಕನ್ನಡ)** - Regional Indian language
- **Spanish (Español)** - International
- **Portuguese (Português)** - International
- **French (Français)** - International
- **German (Deutsch)** - International
- **Italian (Italiano)** - International
- **Russian (Русский)** - International
- **Japanese (日本語)** - International
- **Korean (한국어)** - International
- **Chinese (中文)** - International
- **Arabic (العربية)** - International
- **Turkish (Türkçe)** - International
- **Indonesian (Bahasa Indonesia)** - International
- **Dutch (Nederlands)** - International
- **Polish (Polski)** - International

### Example Commands in Different Languages

**English:**
- "Add a new lead named Rahul"
- "Show me today's deals"

**Hindi:**
- "एक नया लीड जोड़ो राहुल नाम का"
- "आज के डील दिखाओ"

**Hinglish:**
- "ek lead add karo Rahul"
- "aaj ke deals dikhao"

**Gujarati:**
- "રાહુલ નામનો નવો લીડ ઉમેરો"
- "આજના ડીલ બતાવો"

**Spanish:**
- "Agregar un nuevo prospecto llamado Rahul"
- "Mostrar las ofertas de hoy"
---

## Available TTS Voices

### Popular Voices by Language

#### English - US (15 voices)
| Voice ID | Name | Gender | Quality |
|----------|------|--------|---------|
| en_US-lessac-medium | Lessac | Female | Medium |
| en_US-lessac-high | Lessac (HQ) | Female | High |
| en_US-amy-medium | Amy | Female | Medium |
| en_US-ryan-medium | Ryan | Male | Medium |
| en_US-ryan-high | Ryan (HQ) | Male | High |
| en_US-joe-medium | Joe | Male | Medium |

#### English - UK (12 voices)
| Voice ID | Name | Gender | Quality |
|----------|------|--------|---------|
| en_GB-alan-medium | Alan | Male | Medium |
| en_GB-alba-medium | Alba | Female | Medium |
| en_GB-cori-medium | Cori | Female | Medium |
| en_GB-cori-high | Cori (HQ) | Female | High |

#### Hindi (2 voices)
| Voice ID | Name | Gender | Quality |
|----------|------|--------|---------|
| hi_IN-swara-medium | Swara | Female | Medium |
| hi_IN-swara-low | Swara (Low) | Female | Low |

#### German (10 voices)
| Voice ID | Name | Gender | Quality |
|----------|------|--------|---------|
| de_DE-thorsten-medium | Thorsten | Male | Medium |
| de_DE-thorsten-high | Thorsten (HQ) | Male | High |
| de_DE-eva_k-x_low | Eva K | Female | X-Low |

#### Other Languages
- **French (fr-FR)**: 6 voices
- **Spanish (es-ES, es-MX)**: 7 voices
- **Italian (it-IT)**: 2 voices
- **Portuguese (pt-BR, pt-PT)**: 3 voices
- **Dutch (nl-NL, nl-BE)**: 5 voices
- **Polish (pl-PL)**: 4 voices
- **Russian (ru-RU)**: 4 voices
- **Chinese (zh-CN)**: 2 voices
- **Japanese (ja-JP)**: 1 voice
- **Korean (ko-KR)**: 1 voice
- **Arabic (ar-JO)**: 2 voices
- **Turkish (tr-TR)**: 3 voices

**Total: 116+ voices across 40+ languages**

---

## Error Handling

### Standard Error Response Format
```json
{
  "error": "Error message description",
  "error_code": "ERROR_CODE",
  "details": {
    "field_name": ["Specific field error"]
  }
}
```

### Common HTTP Status Codes
| Code | Description | When It Occurs |
|------|-------------|----------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request data, validation errors |
| 401 | Unauthorized | Invalid/missing authentication token |
| 403 | Forbidden | Insufficient permissions, access denied |
| 404 | Not Found | Resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service temporarily unavailable |

### Common Error Scenarios

#### Authentication Errors
```json
{
  "detail": "Authentication credentials were not provided."
}
```

#### Validation Errors
```json
{
  "email": ["This field is required."],
  "password": ["Password must be at least 6 characters long."]
}
```

#### Rate Limiting
```json
{
  "error": "Rate limit exceeded. Try again later.",
  "retry_after_seconds": 3600
}
```

#### TTS Errors
```json
{
  "error": "Text is required"
}
```

#### Voice Agent Errors
```json
{
  "error": "Audio or text input required"
}
```

---

## Rate Limits

### Authentication APIs
- **Login attempts**: 5 failed attempts → 60 minute lockout per email+IP
- **Signup**: No strict limit (fair use policy)

### AI Chat APIs
- **Authenticated users**: No strict limit (fair use policy)
- **Public AI chat**: Configurable per access key (default: 30/min, 1000/day)

### TTS APIs
- **Authenticated users**: No strict limit
- **Public TTS**: Rate limited per access key

### Voice Agent APIs
- **Rate limit**: Configurable per access key

### Lead Capture APIs
- **Public lead capture**: 30 requests per hour per IP
- **Universal lead capture**: Configurable per API key (default: 100/hour)

### WhatsApp APIs
- **Message sending**: Daily limits per account (configurable)
- **API calls**: No strict limit (fair use policy)
---

## Desktop Overlay Integration Tips

For building autonomous desktop overlay applications:

### 1. Authentication Flow
```javascript
// Store tokens securely
const storeTokens = (accessToken, refreshToken) => {
  // Use secure storage (Windows Credential Manager, macOS Keychain, etc.)
  secureStorage.setItem('neen_access_token', accessToken);
  secureStorage.setItem('neen_refresh_token', refreshToken);
};

// Auto-refresh tokens
const refreshTokenIfNeeded = async () => {
  const token = secureStorage.getItem('neen_access_token');
  if (isTokenExpired(token)) {
    const refreshToken = secureStorage.getItem('neen_refresh_token');
    const newTokens = await refreshAccessToken(refreshToken);
    storeTokens(newTokens.access, refreshToken);
  }
};
```

### 2. AI Chat Integration
```javascript
// Natural language command processing
const processCommand = async (userInput) => {
  const response = await fetch('https://api.9ance.com/api/ai/chat/', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      message: userInput,
      conversation_id: currentConversationId,
      model: 'openai/gpt-4o-mini'
    })
  });

  const result = await response.json();
  
  // Execute system actions based on AI response
  if (result.action) {
    await executeSystemAction(result.action, result.action_result);
  }
  
  return result.message;
};

// Map AI actions to system operations
const executeSystemAction = async (action, actionResult) => {
  switch (action.type) {
    case 'create_lead':
      // Integrate with CRM system
      await createLeadInSystem(actionResult.data);
      break;
    case 'list_leads':
      // Display leads in overlay UI
      await displayLeadsOverlay(actionResult.data);
      break;
    case 'send_whatsapp':
      // Send WhatsApp message
      await sendWhatsAppMessage(action.data);
      break;
  }
};
```

### 3. Voice Integration
```javascript
// Voice command processing
const processVoiceCommand = async (audioBlob) => {
  const audioBase64 = await blobToBase64(audioBlob);
  
  const response = await fetch('https://api.9ance.com/api/voice-agent/process/', {
    method: 'POST',
    headers: {
      'X-AI-Access-Key': aiAccessKey,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      audio: audioBase64,
      encoding: 'LINEAR16',
      sample_rate: 16000,
      language: 'en-US',
      voice: 'en_US-lessac-medium',
      return_json: true
    })
  });

  const result = await response.json();
  
  // Play audio response
  if (result.audio_base64) {
    await playAudioResponse(result.audio_base64);
  }
  
  return result;
};
```

### 4. Multi-language Support
```javascript
// Detect user language and adapt
const detectLanguage = (text) => {
  // Implement language detection logic
  if (/[\u0900-\u097F]/.test(text)) return 'hi-IN'; // Hindi
  if (/[\u0A80-\u0AFF]/.test(text)) return 'gu-IN'; // Gujarati
  return 'en-US'; // Default to English
};

// Use appropriate TTS voice
const getVoiceForLanguage = (language) => {
  const voiceMap = {
    'en-US': 'en_US-lessac-medium',
    'hi-IN': 'hi_IN-swara-medium',
    'gu-IN': 'gu_IN-gujarati-medium',
    'es-ES': 'es_ES-davefx-medium'
  };
  return voiceMap[language] || 'en_US-lessac-medium';
};
```

### 5. System Integration
```javascript
// Integrate with system operations
const systemIntegrations = {
  // File operations
  openFile: (filePath) => {
    // Use system APIs to open files
    shell.openPath(filePath);
  },
  
  // Application control
  openApplication: (appName) => {
    // Launch applications
    exec(`start ${appName}`);
  },
  
  // Notification system
  showNotification: (title, message) => {
    new Notification(title, { body: message });
  },
  
  // Screen capture
  captureScreen: async () => {
    // Capture screenshot for AI analysis
    return await desktopCapturer.getSources({ types: ['screen'] });
  }
};
```

### 6. Error Handling & Retry Logic
```javascript
// Robust error handling
const apiCall = async (endpoint, options, retries = 3) => {
  for (let i = 0; i < retries; i++) {
    try {
      const response = await fetch(endpoint, options);
      
      if (response.status === 401) {
        // Token expired, refresh and retry
        await refreshTokenIfNeeded();
        options.headers.Authorization = `Bearer ${newAccessToken}`;
        continue;
      }
      
      if (response.status === 429) {
        // Rate limited, wait and retry
        const retryAfter = response.headers.get('Retry-After') || 60;
        await sleep(retryAfter * 1000);
        continue;
      }
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      return await response.json();
    } catch (error) {
      if (i === retries - 1) throw error;
      await sleep(1000 * Math.pow(2, i)); // Exponential backoff
    }
  }
};
```

### 7. Offline Support
```javascript
// Queue operations when offline
const operationQueue = [];

const queueOperation = (operation) => {
  operationQueue.push({
    ...operation,
    timestamp: Date.now()
  });
};

// Sync when online
const syncQueuedOperations = async () => {
  while (operationQueue.length > 0) {
    const operation = operationQueue.shift();
    try {
      await executeOperation(operation);
    } catch (error) {
      // Re-queue if failed
      operationQueue.unshift(operation);
      break;
    }
  }
};

// Monitor network status
window.addEventListener('online', syncQueuedOperations);
```

---

## Code Examples

### Python Example - AI Chat
```python
import requests
import json

class NeenAIClient:
    def __init__(self, base_url, access_token):
        self.base_url = base_url
        self.access_token = access_token
        self.headers = {
            'Authorization': f'Bearer {access_token}',
            'Content-Type': 'application/json'
        }
    
    def chat(self, message, conversation_id=None, model='openai/gpt-4o-mini'):
        url = f'{self.base_url}/ai/chat/'
        data = {
            'message': message,
            'model': model
        }
        if conversation_id:
            data['conversation_id'] = conversation_id
        
        response = requests.post(url, headers=self.headers, json=data)
        return response.json()
    
    def synthesize_speech(self, text, voice='en_US-lessac-medium'):
        url = f'{self.base_url}/tts/synthesize/'
        data = {
            'text': text,
            'voice': voice,
            'speed': 1.0
        }
        
        response = requests.post(url, headers=self.headers, json=data)
        return response.content  # WAV audio bytes

# Usage
client = NeenAIClient('https://api.9ance.com/api', 'your-access-token')
result = client.chat('Add a new lead named John Doe')
print(result['message'])

# Generate speech
audio_data = client.synthesize_speech('Hello, lead has been added successfully!')
with open('response.wav', 'wb') as f:
    f.write(audio_data)
```

---

## Support & Resources

### API Support
- **Email**: support@9ance.com
- **Documentation**: https://docs.9ance.com

### Rate Limit Increases
Contact support for higher rate limits on:
- AI chat requests
- TTS synthesis
- Voice agent processing
- Lead capture volume

### Custom Integration Support
For complex desktop overlay integrations, NeeN AI team provides:
- Custom API endpoints
- Webhook configurations
- Integration consulting
- Technical support

---

## Changelog

### Version 1.0 (March 2026)
- Initial comprehensive API documentation
- Authentication APIs
- NeeN AI Chat APIs with CRM actions
- Text-to-Speech APIs with 116+ voices
- Voice Agent APIs with STT+AI+TTS pipeline
- Lead Capture APIs (public and authenticated)
- WhatsApp Integration APIs
- Multi-language support (21+ languages)
- Desktop overlay integration guidelines

---

**End of Documentation**

This comprehensive API documentation provides everything needed to build powerful autonomous desktop overlay applications that can interact with NeeN AI's full capabilities including natural language processing, voice commands, CRM operations, and multi-platform integrations.
