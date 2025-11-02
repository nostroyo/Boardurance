# 🚀 Quick Start Guide - Web3 Racing Game

## Prerequisites

Before starting, ensure you have:

- **Docker Desktop** - [Download here](https://www.docker.com/products/docker-desktop)
- **Rust** - [Install here](https://rustup.rs/)
- **Node.js** (v18+) - [Download here](https://nodejs.org/)
- **Git** - [Download here](https://git-scm.com/)

## 🏁 One-Command Startup

To start the entire stack (Docker + MongoDB + Rust Backend + React Frontend):

```powershell
.\start-full-stack.ps1
```

This will:
1. 🐳 Start Docker Desktop (if not running)
2. 🍃 Start MongoDB container
3. 🦀 Build and start Rust backend on port 3000
4. ⚛️ Start React frontend on port 5173
5. 🧪 Run integration tests

## 🛑 Stop Everything

```powershell
.\stop-full-stack.ps1
```

To keep Docker containers running:
```powershell
.\stop-full-stack.ps1 -KeepDocker
```

## 🔗 Service URLs

Once started, you can access:

- **🌐 Frontend**: http://localhost:5173
- **🦀 Backend API**: http://localhost:3000
- **📚 API Documentation**: http://localhost:3000/swagger-ui
- **🍃 MongoDB**: mongodb://localhost:27017

## 🧪 Testing the Authentication Flow

1. **Open the frontend**: http://localhost:5173
2. **Register a new account**:
   - Click "Create Account"
   - Fill out email, password, and team name
   - You'll be automatically logged in
3. **Test the features**:
   - Navigate to the Team page (protected route)
   - Try logging out and logging back in
   - Test authentication persistence (refresh the page)
   - Check error handling with invalid credentials

## 🔧 Individual Component Startup

### Start Only Docker + MongoDB
```powershell
cd rust-backend
.\Makefile.ps1 start-docker
```

### Start Only Backend
```powershell
cd rust-backend
.\Makefile.ps1 dev
```

### Start Only Frontend
```powershell
cd empty-project
npm run dev
```

## 🧪 Running Tests

### Backend Tests
```powershell
cd rust-backend
.\tests\run-all-tests.ps1
```

### Frontend Tests
```powershell
cd empty-project
.\test-frontend-auth.ps1
```

### Integration Tests
```powershell
# Start backend first, then:
cd empty-project
.\test-frontend-auth.ps1
```

## 🔍 Troubleshooting

### Docker Issues
- Ensure Docker Desktop is installed and running
- Try restarting Docker Desktop
- Check if ports 27017 are available

### Backend Issues
- Check if port 3000 is available
- Ensure MongoDB is running
- Check Rust installation: `cargo --version`

### Frontend Issues
- Check if port 5173 is available
- Ensure Node.js is installed: `node --version`
- Try clearing npm cache: `npm cache clean --force`

### Port Conflicts
If you get port conflicts, you can check what's using the ports:
```powershell
netstat -ano | findstr :3000
netstat -ano | findstr :5173
netstat -ano | findstr :27017
```

## 🎯 What's Included

### 🔐 Authentication System
- JWT-based authentication with HTTP-only cookies
- User registration and login
- Automatic token refresh
- Protected routes
- Role-based access control (Player/Admin)

### 🦀 Rust Backend
- RESTful API with OpenAPI documentation
- MongoDB integration
- JWT middleware
- Session management
- Comprehensive error handling

### ⚛️ React Frontend
- Modern React with TypeScript
- Tailwind CSS styling
- Authentication context
- Protected routes
- Error notifications
- Responsive design

### 🧪 Testing Suite
- Integration tests
- Authentication flow tests
- API endpoint tests
- Security edge case tests

## 📋 Next Steps

After starting the application:

1. **Explore the API**: Visit http://localhost:3000/swagger-ui
2. **Test Authentication**: Register and login through the frontend
3. **Check the Code**: Browse the source code to understand the architecture
4. **Run Tests**: Execute the test suites to verify everything works
5. **Customize**: Start building your own features!

## 🆘 Need Help?

- Check the logs in the terminal where you ran the startup script
- Look at the individual component README files
- Check the API documentation at http://localhost:3000/swagger-ui
- Review the test files for usage examples

Happy coding! 🚀