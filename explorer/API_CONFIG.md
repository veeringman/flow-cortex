# FlowCortex Explorer - API Configuration Guide

## Problem Solved
The explorer was hardcoded to connect to `http://127.0.0.1:3000`, which doesn't work in all environments (Docker containers, remote servers, etc.).

## Solution Implemented

### 1. **Dynamic API Configuration**
The API base URL is now fully configurable:

**Methods to set the API endpoint:**

#### A) Via UI (Recommended)
1. Click the **"API Config"** button in the sidebar footer
2. Enter your API endpoint (e.g., `http://192.168.1.100:3000`)
3. Click **"Save & Test"**
4. Configuration is saved to browser localStorage

#### B) Via Browser Console
```javascript
// Set API base dynamically
window.setApiBase('http://your-api-host:3000');
```

#### C) Via HTML Attribute
```html
<body data-api-base="http://your-api-host:3000">
```

#### D) Via JavaScript Global
```javascript
window.API_BASE = 'http://your-api-host:3000';
```

### 2. **File Changes Made**

#### `static/js/modules/api.js`
- Made `API_BASE` dynamic instead of hardcoded constant
- Added `setApiBase()` function to change it at runtime
- Checks localStorage, HTML attribute, and window variable

#### `templates/index.html`
- Added "API Config" button in sidebar
- Added API Configuration modal dialog
- Updated API endpoint display to be dynamic

#### `static/js/app.js`
- Added `updateApiBase()` function to handle config updates
- Loads saved API base from localStorage on startup
- Exports `updateApiBase` function globally

### 3. **Configuration Flow**

```
User clicks "API Config"
         ↓
Click "Save & Test"
         ↓
updateApiBase() called
         ↓
setApiBase() updates API module
         ↓
Saved to localStorage
         ↓
Network status updated
         ↓
Success toast shown
```

## Usage Examples

### Example 1: Local Development (Docker)
```
API Config dialog → Enter: http://192.168.29.78:3000
```

### Example 2: Remote Server
```
API Config dialog → Enter: http://192.168.1.100:3000
```

### Example 3: Cloud Deployment
```
API Config dialog → Enter: https://api.flowcortex.example.com
```

### Example 4: Using Console Command
```javascript
// In browser console:
window.setApiBase('http://my-api-server:3000');
```

## Persistence
- Configuration is **automatically saved** to browser localStorage
- Survives page refreshes
- Can be overwritten anytime via the UI or console

## Testing the Connection
1. Go to **"API Config"** dialog
2. Enter your endpoint
3. Click **"Save & Test"**
4. Check the **network status indicator** (green = connected, red = offline)
5. Try navigating to another tab to test API calls

## Fallback Behavior
If no configuration is provided:
- Default: `http://127.0.0.1:3000`
- Check browser console for API error messages
- Use "API Config" button to change it

## Troubleshooting

### "Offline" Status
1. Verify the API endpoint is correct
2. Check that the L1 node is running on that address
3. Verify network connectivity
4. Check browser console for detailed error messages

### CORS Issues
If you get CORS errors:
1. Make sure the L1 node allows requests from the explorer origin
2. Configure proper CORS headers on the L1 node

### Connection Reset
1. Verify the API endpoint has the correct hostname/IP
2. Verify the port is correct
3. Check that the L1 node is actually listening on that port

## API Endpoints Expected

The explorer expects these endpoints on the configured API base:

```
GET    /blocks                          - List all blocks
POST   /block                           - Create new block
GET    /balance/{account}/{token}       - Get account balance
GET    /pool                            - Get transaction pool
POST   /tx                              - Submit transaction
GET    /snapshot                        - Get pool snapshot
POST   /capsule                         - Upload capsule
GET    /capsule                         - List capsules
POST   /capsule/{id}/invoke             - Invoke capsule
GET    /anchors                         - List anchors
GET    /anchor/{id}                     - Get anchor
POST   /anchor                          - Submit anchor
POST   /token/create                    - Create token
GET    /tokens                          - List tokens
GET    /token/{symbol}                  - Get token metadata
```

## Summary

✅ **Flexible API Configuration**  
✅ **Persistent Storage** (localStorage)  
✅ **Easy UI Configuration**  
✅ **Console Override Support**  
✅ **Network Status Feedback**  
✅ **Error Handling & Toasts**  

Now you can connect the explorer to any FlowCortex L1 node regardless of where it's running!
