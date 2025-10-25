# Implementation Plan

- [ ] 1. Set up project structure and core interfaces
  - Create directory structure for JWT service, middleware, and session management
  - Define core traits and interfaces for authentication and authorization
  - Add required dependencies to Cargo.toml (jsonwebtoken, tower, axum-extra)
  - _Requirements: 1.1, 1.4_

- [ ] 2. Implement JWT service with token generation and validation
  - [ ] 2.1 Create JWT configuration and service structure
    - Implement JwtConfig with environment variable loading
    - Create JwtService with encoding/decoding keys
    - Define Claims structure with user context
    - _Requirements: 1.1, 4.1_

  - [ ] 2.2 Implement token generation methods
    - Add generate_access_token method with proper claims
    - Add generate_refresh_token method with extended expiry
    - Include security claims (iss, aud, jti) for token validation
    - _Requirements: 1.1, 4.1_

  - [ ] 2.3 Implement token validation and refresh
    - Add validate_token method with signature verification
    - Implement token refresh logic with blacklist checking
    - Add proper error handling for expired/invalid tokens
    - _Requirements: 1.3, 1.5, 4.3_

  - [ ] 2.4 Write unit tests for JWT service
    - Test token generation with different user roles
    - Test token validation with valid/invalid/expired tokens
    - Test token refresh functionality and error cases
    - _Requirements: 1.1, 1.3, 1.5_

- [ ] 3. Create session management with MongoDB and in-memory caching
  - [ ] 3.1 Implement session data structures and MongoDB collections
    - Create Session and BlacklistedToken models
    - Set up MongoDB collections with proper indexes
    - Define SessionConfig with cache size limits
    - _Requirements: 3.1, 3.2_

  - [ ] 3.2 Implement session manager with dual storage
    - Create SessionManager with MongoDB and in-memory cache
    - Implement session creation and validation methods
    - Add token blacklisting with automatic expiration
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ] 3.3 Add session cleanup and cache management
    - Implement expired session cleanup background task
    - Add cache synchronization with MongoDB
    - Implement LRU cache eviction for memory management
    - _Requirements: 3.2, 3.4_

  - [ ] 3.4 Write unit tests for session management
    - Test session creation and validation
    - Test token blacklisting functionality
    - Test cache synchronization and cleanup
    - _Requirements: 3.1, 3.2, 3.3_

- [ ] 4. Implement authentication middleware
  - [ ] 4.1 Create authentication middleware structure
    - Implement AuthMiddleware as Tower Layer
    - Create UserContext for storing authenticated user data
    - Define AuthError enum with proper error handling
    - _Requirements: 1.2, 1.3, 5.1_

  - [ ] 4.2 Implement token extraction from requests
    - Extract JWT tokens from Authorization header
    - Extract JWT tokens from HTTP-only cookies
    - Implement fallback logic for different client types
    - _Requirements: 1.2, 4.2_

  - [ ] 4.3 Add middleware request processing
    - Validate extracted tokens using JWT service
    - Check token blacklist status via session manager
    - Attach UserContext to request extensions
    - _Requirements: 1.2, 1.3, 1.4_

  - [ ] 4.4 Write unit tests for authentication middleware
    - Test middleware with valid/invalid tokens
    - Test cookie and header token extraction
    - Test error response formatting
    - _Requirements: 1.2, 1.3, 5.1_

- [ ] 5. Implement smart authorization middleware with ownership validation
  - [ ] 5.1 Create RequireOwnership middleware structure
    - Implement RequireOwnership as configurable Tower Layer
    - Create OwnershipValidationType enum for different validation patterns
    - Add factory methods for common ownership patterns
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ] 5.2 Implement path parameter extraction and validation
    - Create helper functions to extract UUIDs from request paths
    - Implement player ownership validation logic
    - Add admin bypass for all ownership checks
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ] 5.3 Add role-based authorization middleware
    - Implement RequireRole middleware for admin-only endpoints
    - Create factory methods for common role requirements
    - Add proper error responses for authorization failures
    - _Requirements: 2.1, 2.3, 5.2_

  - [ ] 5.4 Write unit tests for authorization middleware
    - Test ownership validation with different user scenarios
    - Test role-based access control
    - Test error responses for authorization failures
    - _Requirements: 2.1, 2.2, 2.3_

- [ ] 6. Update authentication routes with JWT token generation
  - [ ] 6.1 Enhance login endpoint with JWT token generation
    - Integrate JWT service into existing login route
    - Generate access and refresh tokens on successful login
    - Set secure HTTP-only cookies for web clients
    - _Requirements: 1.1, 4.2_

  - [ ] 6.2 Add token refresh endpoint
    - Create refresh token endpoint for token renewal
    - Validate refresh tokens and issue new access tokens
    - Implement proper cookie management for refresh flow
    - _Requirements: 1.5, 4.2_

  - [ ] 6.3 Implement logout endpoint with session invalidation
    - Create logout endpoint that invalidates current session
    - Clear HTTP-only cookies on logout
    - Add session cleanup in session manager
    - _Requirements: 3.1, 4.2_

  - [ ] 6.4 Write integration tests for auth endpoints
    - Test complete login flow with token generation
    - Test token refresh workflow
    - Test logout and session invalidation
    - _Requirements: 1.1, 1.5, 3.1_

- [ ] 7. Apply middleware to existing routes with ownership protection
  - [ ] 7.1 Protect player routes with ownership validation
    - Apply RequireOwnership::player middleware to player endpoints
    - Remove manual ownership checks from player handlers
    - Update handlers to use guaranteed-valid UserContext
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ] 7.2 Protect race routes with participation validation
    - Apply custom ownership validation for race endpoints
    - Implement race participation checking logic
    - Add admin bypass for race management endpoints
    - _Requirements: 2.1, 2.2, 2.5_

  - [ ] 7.3 Add admin-only routes protection
    - Apply RequireRole::admin to administrative endpoints
    - Create admin route group with centralized protection
    - Update route organization for clear security boundaries
    - _Requirements: 2.1, 2.3_

  - [ ] 7.4 Write integration tests for protected routes
    - Test player accessing own resources (allowed)
    - Test player accessing other player resources (blocked)
    - Test admin accessing any resources (allowed)
    - _Requirements: 2.1, 2.2, 2.3_

- [ ] 8. Add user roles to Player domain model
  - [ ] 8.1 Extend Player model with role field
    - Add UserRole enum to domain model
    - Update Player struct with role field and helper methods
    - Add role field to database schema and migrations
    - _Requirements: 2.1, 2.3_

  - [ ] 8.2 Update player creation with default roles
    - Set default role to Player for new registrations
    - Update existing player creation logic
    - Add role validation in player domain logic
    - _Requirements: 2.1, 2.3_

  - [ ] 8.3 Write unit tests for role functionality
    - Test role assignment and validation
    - Test role-based helper methods
    - Test database operations with roles
    - _Requirements: 2.1, 2.3_

- [ ] 9. Create comprehensive integration tests
  - [ ] 9.1 Test complete authentication flow
    - Test registration → login → protected route access
    - Test token refresh and session management
    - Test logout and session cleanup
    - _Requirements: 1.1, 1.5, 3.1_

  - [ ] 9.2 Test authorization scenarios
    - Test ownership validation across different endpoints
    - Test role-based access control
    - Test admin privileges and bypasses
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ] 9.3 Test security edge cases
    - Test token tampering and invalid signatures
    - Test expired token handling
    - Test blacklisted token rejection
    - _Requirements: 1.3, 3.2, 4.3_

- [ ] 10. Frontend authentication integration
  - [ ] 10.1 Update frontend auth service for cookie-based authentication
    - Modify login/logout functions to work with cookies
    - Implement automatic token refresh on 401 responses
    - Add proper error handling for authentication failures
    - _Requirements: 1.1, 1.5, 5.1_

  - [ ] 10.2 Update frontend components to use new auth flow
    - Update login and registration components
    - Add authentication state management
    - Implement protected route components
    - _Requirements: 1.1, 5.1_

  - [ ] 10.3 Test frontend authentication integration
    - Test login/logout flow with backend
    - Test automatic token refresh
    - Test protected route access
    - _Requirements: 1.1, 1.5, 5.1_