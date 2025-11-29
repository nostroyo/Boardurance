import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { PlayerGameProvider } from './contexts/PlayerGameContext';
import ErrorNotification from './components/ErrorNotification';
import MainPage from './components/MainPage';
import RegistrationPage from './components/RegistrationPage';
import LoginPage from './components/LoginPage';
import Dashboard from './components/Dashboard';
import TeamPage from './components/TeamPage';
import ProtectedRoute from './components/ProtectedRoute';
import AdminRoute from './components/AdminRoute';
import AdminDashboard from './components/AdminDashboard';
import GameLobby from './components/GameLobby';
import GameWrapper from './components/GameWrapper';
import './App.css';

function App() {
  return (
    <AuthProvider>
      <Router>
        <div className="App">
          <ErrorNotification />
          <Routes>
            <Route path="/" element={<MainPage />} />
            <Route path="/register" element={<RegistrationPage />} />
            <Route path="/login" element={<LoginPage />} />
            <Route 
              path="/dashboard" 
              element={
                <ProtectedRoute>
                  <Dashboard />
                </ProtectedRoute>
              } 
            />
            <Route 
              path="/team" 
              element={
                <ProtectedRoute>
                  <TeamPage />
                </ProtectedRoute>
              } 
            />
            <Route 
              path="/admin" 
              element={
                <AdminRoute>
                  <AdminDashboard />
                </AdminRoute>
              } 
            />
            <Route 
              path="/game/:raceUuid" 
              element={
                <ProtectedRoute>
                  <GameWrapper />
                </ProtectedRoute>
              } 
            />
            <Route 
              path="/game" 
              element={
                <ProtectedRoute>
                  <GameLobby />
                </ProtectedRoute>
              } 
            />
          </Routes>
        </div>
      </Router>
    </AuthProvider>
  );
}

export default App;