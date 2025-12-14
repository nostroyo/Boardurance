---
inclusion: always
---

Name features with a number to easily track changes and documentations

When you start a new feature create a new branch on git with the name of the feature. This will allow you to have a clean working directory and not have to worry about conflicts when you merge your changes back into the main branch.

```sh
git checkout -b feature/my-feature
```
When I finished the end-to-end test and I say that everything is good, you can merge your changes back into the main branch. BUT NEVER by yourself, always wait for my approval

```sh

When you finish a task inside your feature branch always commit with the task number and the name of the task. This will allow you to have a clean working directory and not have to worry about conflicts when you merge your changes back into the feature branch.

```sh
git add .
git commit -m "feat: #1234 Add new feature"
```

All your documentation (ie .md) should be in the docs folder. This will allow you to have a clean working directory and not have to worry about conflicts when you merge your changes back into the main branch.
Try to use sub directory to stay clean in the repository

In the docs folder you can have a folder for each feature. This will allow you to have a clean working directory and not have to worry about conflicts when you merge your changes back into the main branch.

```sh
docs/feature/my-feature
```
When you finish a task inside your feature branch always commit with the task number and the name of the task. This will allow you to have a clean working directory and not have to worry about conflicts when you merge your changes back into the feature branch.

```sh

---
inclusion: always
---

# Development Organization & Standards

## Code Quality Standards

### Rust Backend
- Use `thiserror` for error handling with descriptive error messages
- Implement comprehensive logging with `tracing` and structured JSON output
- Follow domain-driven design patterns with clear separation between domain, routes, and infrastructure layers
- Use `clippy::pedantic` linting level and address all warnings
- Implement proper error propagation using `Result<T, E>` types
- Use `secrecy` crate for sensitive configuration values

### React Frontend
- Use TypeScript strict mode with explicit return types for functions
- Implement proper error boundaries for component error handling
- Use custom hooks for shared logic and state management
- Follow component composition patterns over inheritance
- Implement proper loading and error states for async operations
- Use Tailwind CSS utility classes consistently

### Testing Requirements
- Write integration tests for all API endpoints
- Implement unit tests for domain logic and utility functions
- Use descriptive test names that explain the scenario being tested
- Mock external dependencies in tests (database, blockchain calls)
- Maintain test coverage above 80% for critical business logic

## Architecture Patterns

### API Design
- Follow RESTful conventions with proper HTTP status codes
- Use consistent error response format across all endpoints
- Implement proper request validation with descriptive error messages
- Use OpenAPI specifications for API documentation
- Implement rate limiting and authentication middleware

### State Management
- Use React hooks for local component state
- Implement proper data fetching patterns with loading/error states
- Use optimistic updates for better user experience
- Cache API responses appropriately to reduce server load

### Database Operations
- Use MongoDB aggregation pipelines for complex queries
- Implement proper indexing for performance-critical queries
- Use transactions for operations that modify multiple collections
- Implement soft deletes for audit trail requirements

## Development Workflow

### Code Review Standards
- All code must pass compilation without warnings
- Run linting and formatting tools before committing
- Write clear commit messages following conventional commit format
- Test all changes locally before creating pull requests

### Performance Guidelines
- API responses should be under 100ms for 95th percentile
- Frontend components should render within 16ms for 60fps
- Implement proper pagination for large data sets
- Use connection pooling for database operations

### Security Practices
- Validate all user inputs on both client and server side
- Use HTTP-only cookies for authentication tokens
- Implement proper CORS configuration
- Sanitize data before database operations
- Use environment variables for sensitive configuration