# NeeN Desktop Agent — API Documentation

Base URL: `https://crmapi.9ance.com`  
Auth: Bearer token on all endpoints (except webhooks)

---

## Activity Reminder Notifications — Real-time API

### WebSocket Connection

**URL:**
```
wss://crmapi.9ance.com/ws/activity/notifications/
```
or
```
wss://crmapi.9ance.com/ws/user/<user_id>/activity/notifications/
```

**Auth:** Token passed via query param or cookie (same as WhatsApp WebSocket)

**Keepalive:** Send ping every 30s:
```json
{"type": "ping"}
```

**Response:**
```json
{"type": "pong", "timestamp": "2026-05-25T09:01:12.831575+00:00"}
```

---

### WebSocket Incoming Messages (server → client)

**Activity Reminder Push:**
```json
{
    "type": "activity_reminder",
    "notification": {
        "id": "notification-uuid",
        "title": "Follow up with Rahul",
        "message": "Follow up with Rahul starts in 15 minutes",
        "priority": "high",
        "reminderLabel": "15 minutes",
        "createdAt": "2026-05-25T12:15:00+05:30",
        "activity": {
            "id": "activity-uuid",
            "type": "call",
            "subject": "Follow up with Rahul",
            "notes": "Discuss pricing",
            "status": "pending",
            "date": "2026-05-25",
            "startTime": "12:30",
            "endTime": "13:00",
            "timeDisplay": "May 25, 2026 from 12:30 PM to 01:00 PM",
            "ownerName": "Vaishali Barskar"
        },
        "relatedEntity": {
            "type": "Lead",
            "id": "lead-uuid",
            "name": "Rahul Sharma",
            "email": "rahul@example.com",
            "phone": "9876543210",
            "company": "Acme Corp",
            "status": "qualified",
            "source": "Website",
            "owner": "Vaishali Barskar"
        }
    }
}
```

Reminders fire at **45 min**, **30 min**, and **15 min** before the activity start time.

---

### REST APIs

#### 1. Get Upcoming Reminders (next 24 hours)

```
GET /api/activity-reminders/upcoming/
```

**Response:**
```json
{
    "count": 2,
    "results": [
        {
            "id": "activity-uuid",
            "type": "call",
            "subject": "Follow up with Rahul",
            "date": "2026-05-25T12:30:00+05:30",
            "start_time": "12:30:00",
            "end_time": "13:00:00",
            "minutes_until_start": 42,
            "reminders": [
                {"minutes": 15, "label": "15 minutes", "sent": false, "applicable": false},
                {"minutes": 30, "label": "30 minutes", "sent": false, "applicable": false},
                {"minutes": 45, "label": "45 minutes", "sent": false, "applicable": true}
            ]
        }
    ],
    "reminder_intervals": [
        {"minutes": 15, "label": "15 minutes"},
        {"minutes": 30, "label": "30 minutes"},
        {"minutes": 45, "label": "45 minutes"}
    ]
}
```

#### 2. Reminder History

```
GET /api/activity-reminders/history/?limit=20&offset=0
```

#### 3. Acknowledge Reminder

```
POST /api/activity-reminders/<activity_id>/acknowledge/
```

**Payload:**
```json
{"action": "accepted"}
```

Options: `accepted`, `declined`, `snoozed`

**For snooze:**
```json
{"action": "snoozed", "snooze_minutes": 5}
```

**Response:**
```json
{
    "success": true,
    "action": "snoozed",
    "activity_id": "uuid",
    "next_reminder_at": "2026-05-25T12:35:00+05:30",
    "message": "Snoozed for 5 minutes"
}
```

#### 4. Manually Trigger Reminder Check (admin/debug)

```
POST /api/activity-reminders/check/
```

---

### Flow

1. Celery beat runs `check_and_send_reminders()` every minute
2. Finds activities with `status=pending` starting within 45 min
3. Sends notifications at 45, 30, 15 min intervals
4. Channels: WebSocket push → `activity_reminder` event (for popup)
5. FCM: Push notification for mobile (15 min = call-type with native ring UI)
6. WhatsApp: Reminder to both owner and customer/lead

---

## Telephony API

Base URL: `https://crmapi.9ance.com/api/telephony/`

### 1. Configuration

```
GET /api/telephony/config/
POST /api/telephony/config/
POST /api/telephony/config/test/
DELETE /api/telephony/config/
```

### 2. Click-to-Call

```
POST /api/telephony/calls/click-to-call/
```

**Payload:**
```json
{
    "agent_id": "uuid-of-telephony-agent",
    "customer_number": "9876543210",
    "did_id": "uuid-of-did (optional)",
    "wait_seconds": 30,
    "lead_id": "uuid (optional)",
    "contact_id": "uuid (optional)",
    "customer_id": "uuid (optional)"
}
```

### 3. Softphone Config (WebRTC)

```
GET /api/telephony/agents/me/softphone-config/
```

**Response:**
```json
{
    "enabled": true,
    "agent_id": "agent-uuid",
    "extension": "101",
    "sip": {
        "username": "07415912090",
        "password": "07415912090",
        "display_name": "Agent Name",
        "uri": "sip:07415912090@kenyavoice.rpdigitalphone.com"
    },
    "ws": {
        "url": "ws://kenyavoice.rpdigitalphone.com:5000/ws",
        "transport": "ws"
    },
    "ice": {
        "stun": ["stun:stun.l.google.com:19302"],
        "turn": []
    }
}
```

### 4. Live Calls

```
GET /api/telephony/calls/live/
```

### 5. Phone Match (Screen Pop)

```
GET /api/telephony/calls/match/?phone=9876543210
```

### 6. Call History

```
GET /api/telephony/calls/?direction=outbound&agent_id=<uuid>&since=2026-05-01&until=2026-05-25
POST /api/telephony/calls/sync/
```

### 7. Agents

```
GET /api/telephony/agents/
GET /api/telephony/agents/me/
GET /api/telephony/agents/me/status/
POST /api/telephony/agents/me/status/
```

### 8. DIDs

```
GET /api/telephony/dids/
POST /api/telephony/dids/sync/
POST /api/telephony/dids/<did-id>/set-default/
```

### 9. SIM-Based Call Relay

```
POST /api/calls/relay/
```

```json
{
    "phone_number": "9876543210",
    "contact_name": "Rahul Sharma",
    "lead_id": "uuid (optional)",
    "enable_recording": true
}
```
