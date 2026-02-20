# Contract Maintenance Mode - Implementation Summary

## Overview
Implemented a complete maintenance mode feature for Soroban Registry contracts, allowing publishers to temporarily put contracts in read-only mode with custom messages and scheduled automatic restarts.

## Files Created

### Backend
1. **database/migrations/004_maintenance_mode.sql**
   - Creates `maintenance_windows` table
   - Adds `is_maintenance` flag to contracts table
   - Indexes for performance

2. **backend/api/src/maintenance_handlers.rs**
   - `start_maintenance()` - PUT contract in maintenance mode
   - `end_maintenance()` - Exit maintenance mode
   - `get_maintenance_status()` - Check current status
   - `get_maintenance_history()` - View all past maintenance windows

3. **backend/api/src/maintenance_middleware.rs**
   - Intercepts write operations (POST/PUT/PATCH/DELETE)
   - Returns 503 with custom message during maintenance
   - Allows read operations (GET) to continue

4. **backend/api/src/maintenance_routes.rs**
   - Routes for maintenance endpoints

5. **backend/api/src/maintenance_scheduler.rs**
   - Background task running every 60 seconds
   - Automatically ends maintenance at scheduled time

6. **backend/api/src/maintenance_tests.rs**
   - Test structure for maintenance features

### Shared Models
7. **backend/shared/src/models.rs** (modified)
   - Added `is_maintenance` field to `Contract` struct
   - Added `MaintenanceWindow` struct
   - Added `StartMaintenanceRequest` struct
   - Added `MaintenanceStatusResponse` struct

### Frontend
8. **frontend/lib/api.ts** (modified)
   - Added `is_maintenance` to Contract interface
   - Added `MaintenanceWindow` interface
   - Added `MaintenanceStatus` interface
   - Added `maintenanceApi` with CRUD operations

9. **frontend/components/MaintenanceBanner.tsx**
   - Yellow warning banner component
   - Shows maintenance message and scheduled end time

10. **frontend/app/contracts/[id]/page.tsx** (modified)
    - Queries maintenance status
    - Displays MaintenanceBanner when active

### Documentation
11. **docs/MAINTENANCE_MODE.md**
    - Complete feature documentation
    - API examples
    - Integration guide

## API Endpoints

```
POST   /api/contracts/:id/maintenance       - Start maintenance
DELETE /api/contracts/:id/maintenance       - End maintenance
GET    /api/contracts/:id/maintenance       - Get status
GET    /api/contracts/:id/maintenance/history - Get history
```

## Key Features Implemented

✅ **Status Management**: Contracts can be marked maintenance/read-only
✅ **Custom Messages**: Publishers set informative messages for users
✅ **Scheduled Exit**: Automatic maintenance end at specified time
✅ **API Protection**: Returns 503 for write operations during maintenance
✅ **UI Banner**: Yellow warning banner explains maintenance status
✅ **History Logging**: All maintenance windows are tracked and searchable
✅ **Background Scheduler**: Automatic cleanup of expired maintenance windows

## Acceptance Criteria Met

✅ Maintenance mode prevents writes (503 response)
✅ UI shows maintenance message (MaintenanceBanner component)
✅ Scheduled exit happens automatically (maintenance_scheduler)
✅ Users can check status via API (GET endpoint)
✅ Maintenance history is searchable (history endpoint)

## Integration Points

1. **Middleware Layer**: Maintenance check runs before rate limiting
2. **Database**: New table with foreign key to contracts
3. **Background Tasks**: Scheduler spawned on app startup
4. **Frontend**: React Query integration for real-time status

## Usage Example

```bash
# Start maintenance
curl -X POST http://localhost:3001/api/contracts/{id}/maintenance \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Upgrading to v2.0 - back at 3pm UTC",
    "scheduled_end_at": "2026-02-20T15:00:00Z"
  }'

# Check status
curl http://localhost:3001/api/contracts/{id}/maintenance

# End maintenance manually
curl -X DELETE http://localhost:3001/api/contracts/{id}/maintenance
```

## Next Steps

To deploy this feature:

1. Run database migration: `sqlx migrate run`
2. Restart backend API server
3. Deploy frontend with updated components
4. Test with a sample contract

## Notes

- Middleware extracts contract_id from URL path
- Scheduler runs every 60 seconds (configurable)
- Read operations continue during maintenance
- History is preserved indefinitely for audit purposes
