import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { authUtils, apiUtils } from '../utils/auth';

function RegistrationPage() {
  const [formData, setFormData] = useState({
    email: '',
    password: '',
    teamName: '',
    terms: false
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const navigate = useNavigate();

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!formData.terms) {
      setError('Please agree to the Terms and Conditions');
      return;
    }

    setIsLoading(true);

    try {
      const result = await apiUtils.register(formData.email, formData.password, formData.teamName);
      
      if (result.success && result.data) {
        console.log('User registered successfully:', result.data);
        
        // Store user data using auth utility
        authUtils.setCurrentUser({
          uuid: result.data.uuid,
          email: formData.email,
          team_name: formData.teamName
        });
        
        // Navigate to team page
        navigate('/team');
      } else {
        setError(result.error || 'Failed to create account. Please try again.');
      }
    } catch (err) {
      setError('Network error. Please check your connection and try again.');
      console.error('Network error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-900 to-purple-900 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-xl p-8 max-w-md w-full mx-4">
        <div className="text-center mb-6">
          <h1 className="text-2xl font-bold text-gray-800 mb-2">
            Create Your Account
          </h1>
          <p className="text-gray-600">
            Join the Web3 Racing Game community
          </p>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-1">
              Email
            </label>
            <input
              type="email"
              id="email"
              name="email"
              value={formData.email}
              onChange={handleInputChange}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Enter your email"
              required
            />
          </div>

          <div>
            <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-1">
              Password
            </label>
            <input
              type="password"
              id="password"
              name="password"
              value={formData.password}
              onChange={handleInputChange}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Enter your password"
              required
            />
            <p className="text-xs text-gray-500 mt-1">
              Password must be at least 8 characters with uppercase, lowercase, and a number
            </p>
          </div>

          <div>
            <label htmlFor="teamName" className="block text-sm font-medium text-gray-700 mb-1">
              Team Name
            </label>
            <input
              type="text"
              id="teamName"
              name="teamName"
              value={formData.teamName}
              onChange={handleInputChange}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Enter your team name"
              required
            />
          </div>

          <div className="flex items-center">
            <input
              type="checkbox"
              id="terms"
              name="terms"
              checked={formData.terms}
              onChange={handleInputChange}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              required
            />
            <label htmlFor="terms" className="ml-2 block text-sm text-gray-700">
              I agree to the{' '}
              <a href="#" className="text-blue-600 hover:text-blue-500">
                Terms and Conditions
              </a>
            </label>
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 disabled:cursor-not-allowed text-white font-semibold py-3 px-6 rounded-lg transition duration-200"
          >
            {isLoading ? 'Creating Account...' : 'Create Account'}
          </button>
        </form>

        <div className="mt-6 text-center">
          <p className="text-sm text-gray-600">
            Already have an account?{' '}
            <Link to="/login" className="text-blue-600 hover:text-blue-500 font-medium">
              Sign in
            </Link>
          </p>
          <Link
            to="/"
            className="inline-block mt-2 text-sm text-gray-500 hover:text-gray-700"
          >
            ← Back to Home
          </Link>
        </div>
      </div>
    </div>
  );
}

export default RegistrationPage;