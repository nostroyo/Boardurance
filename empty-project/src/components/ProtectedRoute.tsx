import { useEffect } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthContext } from '../contexts/AuthContext';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { isAuthenticated, isLoading, checkAuthStatus } = useAuthContext();
  const location = useLocation();

  useEffect(() => {
    // Check authentication status when component mounts
    checkAuthStatus();
  }, [checkAuthStatus]);

  // Show loading while checking authentication
  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <div className="text-xl text-gray-600">Checking authentication...</div>
        </div>
      </div>
    );
  }

  // Redirect to login if not authenticated
  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Render protected content
  return <>{children}</>;
}

export default ProtectedRoute;