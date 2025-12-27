# 🎨 Exemplos de Frontend - OAuth Integration

Exemplos práticos de integração OAuth com React, Vue e Angular.

---

## 📱 React + TypeScript

### Instalação

```bash
npm install axios
# ou
yarn add axios
```

### 1. OAuth Service (`src/services/auth.service.ts`)

```typescript
import axios from 'axios';

const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';
const TENANT_ID = process.env.REACT_APP_TENANT_ID || 'your-tenant-id';

export interface User {
  id: string;
  email: string;
  name?: string;
  picture?: string;
  roles: string[];
  oauth_provider: 'google' | 'apple';
}

export interface AuthResponse {
  token: string;
  user: User;
}

class AuthService {
  // Obter URL de autorização do Google
  async getGoogleAuthUrl(): Promise<{ auth_url: string; state: string }> {
    const response = await axios.get(`${API_URL}/api/auth/google`, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });
    return response.data;
  }

  // Obter URL de autorização da Apple
  async getAppleAuthUrl(): Promise<{ auth_url: string; state: string }> {
    const response = await axios.get(`${API_URL}/api/auth/apple`, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });
    return response.data;
  }

  // Processar callback (chamado automaticamente pela URL)
  async handleCallback(currentUrl: string): Promise<AuthResponse> {
    const response = await axios.get(currentUrl, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });
    return response.data;
  }

  // Salvar token e usuário
  saveAuth(token: string, user: User) {
    localStorage.setItem('token', token);
    localStorage.setItem('user', JSON.stringify(user));
  }

  // Obter token salvo
  getToken(): string | null {
    return localStorage.getItem('token');
  }

  // Obter usuário salvo
  getUser(): User | null {
    const user = localStorage.getItem('user');
    return user ? JSON.parse(user) : null;
  }

  // Logout
  logout() {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    localStorage.removeItem('oauth_state');
  }

  // Verificar se está autenticado
  isAuthenticated(): boolean {
    return !!this.getToken();
  }

  // Criar instância axios com token
  createAuthAxios() {
    const token = this.getToken();
    return axios.create({
      baseURL: API_URL,
      headers: {
        'Authorization': `Bearer ${token}`,
        'X-Tenant-ID': TENANT_ID
      }
    });
  }
}

export default new AuthService();
```

### 2. Login Component (`src/components/Login.tsx`)

```typescript
import React, { useState } from 'react';
import authService from '../services/auth.service';
import './Login.css';

const Login: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGoogleLogin = async () => {
    try {
      setLoading(true);
      setError(null);

      // 1. Obter URL de autorização
      const { auth_url, state } = await authService.getGoogleAuthUrl();

      // 2. Salvar state para CSRF protection
      localStorage.setItem('oauth_state', state);

      // 3. Redirecionar para Google
      window.location.href = auth_url;
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to login with Google');
      setLoading(false);
    }
  };

  const handleAppleLogin = async () => {
    try {
      setLoading(true);
      setError(null);

      const { auth_url, state } = await authService.getAppleAuthUrl();
      localStorage.setItem('oauth_state', state);
      window.location.href = auth_url;
    } catch (err: any) {
      setError(err.response?.data?.message || 'Failed to login with Apple');
      setLoading(false);
    }
  };

  return (
    <div className="login-container">
      <div className="login-card">
        <h1>Welcome</h1>
        <p>Sign in to continue</p>

        {error && (
          <div className="alert alert-error">
            {error}
          </div>
        )}

        <div className="oauth-buttons">
          <button
            className="btn btn-google"
            onClick={handleGoogleLogin}
            disabled={loading}
          >
            <img src="/icons/google.svg" alt="Google" />
            {loading ? 'Redirecting...' : 'Continue with Google'}
          </button>

          <button
            className="btn btn-apple"
            onClick={handleAppleLogin}
            disabled={loading}
          >
            <img src="/icons/apple.svg" alt="Apple" />
            {loading ? 'Redirecting...' : 'Continue with Apple'}
          </button>
        </div>

        <p className="privacy-note">
          By signing in, you agree to our Terms of Service and Privacy Policy.
        </p>
      </div>
    </div>
  );
};

export default Login;
```

### 3. OAuth Callback Component (`src/components/OAuthCallback.tsx`)

```typescript
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import authService from '../services/auth.service';

const OAuthCallback: React.FC = () => {
  const navigate = useNavigate();
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const handleCallback = async () => {
      try {
        // 1. Obter parâmetros da URL
        const params = new URLSearchParams(window.location.search);
        const code = params.get('code');
        const state = params.get('state');

        if (!code || !state) {
          throw new Error('Invalid callback parameters');
        }

        // 2. Verificar CSRF state
        const savedState = localStorage.getItem('oauth_state');
        if (state !== savedState) {
          throw new Error('Invalid state parameter - possible CSRF attack');
        }

        // 3. Backend já processa o callback automaticamente
        const { token, user } = await authService.handleCallback(window.location.href);

        // 4. Salvar autenticação
        authService.saveAuth(token, user);
        localStorage.removeItem('oauth_state');

        // 5. Redirecionar para dashboard
        setStatus('success');
        setTimeout(() => navigate('/dashboard'), 1000);

      } catch (err: any) {
        console.error('OAuth callback error:', err);
        setStatus('error');
        setError(err.response?.data?.message || err.message || 'Authentication failed');

        // Redirecionar para login após 3 segundos
        setTimeout(() => navigate('/login'), 3000);
      }
    };

    handleCallback();
  }, [navigate]);

  return (
    <div className="callback-container">
      {status === 'processing' && (
        <div className="loading">
          <div className="spinner"></div>
          <p>Completing sign in...</p>
        </div>
      )}

      {status === 'success' && (
        <div className="success">
          <div className="check-icon">✓</div>
          <p>Sign in successful! Redirecting...</p>
        </div>
      )}

      {status === 'error' && (
        <div className="error">
          <div className="error-icon">✗</div>
          <p>Authentication failed</p>
          <p className="error-message">{error}</p>
          <p>Redirecting to login...</p>
        </div>
      )}
    </div>
  );
};

export default OAuthCallback;
```

### 4. Protected Route Component (`src/components/ProtectedRoute.tsx`)

```typescript
import React from 'react';
import { Navigate } from 'react-router-dom';
import authService from '../services/auth.service';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children }) => {
  if (!authService.isAuthenticated()) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
};

export default ProtectedRoute;
```

### 5. App Routes (`src/App.tsx`)

```typescript
import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Login from './components/Login';
import OAuthCallback from './components/OAuthCallback';
import Dashboard from './components/Dashboard';
import ProtectedRoute from './components/ProtectedRoute';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={<Login />} />
        <Route path="/auth/google/callback" element={<OAuthCallback />} />
        <Route path="/auth/apple/callback" element={<OAuthCallback />} />

        {/* Protected routes */}
        <Route
          path="/dashboard"
          element={
            <ProtectedRoute>
              <Dashboard />
            </ProtectedRoute>
          }
        />

        {/* Default redirect */}
        <Route path="*" element={<Navigate to="/dashboard" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
```

### 6. Dashboard Example (`src/components/Dashboard.tsx`)

```typescript
import React, { useEffect, useState } from 'react';
import authService from '../services/auth.service';

const Dashboard: React.FC = () => {
  const [user, setUser] = useState(authService.getUser());

  const handleLogout = () => {
    authService.logout();
    window.location.href = '/login';
  };

  return (
    <div className="dashboard">
      <header>
        <h1>Dashboard</h1>
        <div className="user-info">
          {user?.picture && (
            <img src={user.picture} alt="Profile" className="avatar" />
          )}
          <div>
            <p className="user-name">{user?.name || user?.email}</p>
            <p className="user-email">{user?.email}</p>
          </div>
          <button onClick={handleLogout}>Logout</button>
        </div>
      </header>

      <main>
        <h2>Welcome, {user?.name || 'User'}!</h2>
        <p>You are signed in with {user?.oauth_provider}</p>
      </main>
    </div>
  );
};

export default Dashboard;
```

### 7. CSS Example (`src/components/Login.css`)

```css
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.login-card {
  background: white;
  border-radius: 16px;
  padding: 48px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 400px;
  width: 100%;
}

.login-card h1 {
  font-size: 32px;
  margin-bottom: 8px;
  color: #333;
}

.login-card > p {
  color: #666;
  margin-bottom: 32px;
}

.oauth-buttons {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 14px 24px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn img {
  width: 20px;
  height: 20px;
}

.btn-google {
  background: white;
  color: #333;
  border: 1px solid #ddd;
}

.btn-google:hover:not(:disabled) {
  background: #f8f8f8;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.btn-apple {
  background: #000;
  color: white;
}

.btn-apple:hover:not(:disabled) {
  background: #333;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.privacy-note {
  margin-top: 24px;
  font-size: 12px;
  color: #999;
  text-align: center;
}

.alert {
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 16px;
}

.alert-error {
  background: #fee;
  color: #c00;
  border: 1px solid #fcc;
}
```

---

## 🖼️ Vue 3 + Composition API

### 1. Auth Composable (`src/composables/useAuth.ts`)

```typescript
import { ref, computed } from 'vue';
import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const TENANT_ID = import.meta.env.VITE_TENANT_ID || 'your-tenant-id';

interface User {
  id: string;
  email: string;
  name?: string;
  picture?: string;
  roles: string[];
  oauth_provider: 'google' | 'apple';
}

const user = ref<User | null>(null);
const token = ref<string | null>(null);

export function useAuth() {
  const isAuthenticated = computed(() => !!token.value);

  // Carregar dados salvos
  const loadFromStorage = () => {
    token.value = localStorage.getItem('token');
    const savedUser = localStorage.getItem('user');
    user.value = savedUser ? JSON.parse(savedUser) : null;
  };

  // Login com Google
  const loginWithGoogle = async () => {
    const response = await axios.get(`${API_URL}/api/auth/google`, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });

    const { auth_url, state } = response.data;
    localStorage.setItem('oauth_state', state);
    window.location.href = auth_url;
  };

  // Login com Apple
  const loginWithApple = async () => {
    const response = await axios.get(`${API_URL}/api/auth/apple`, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });

    const { auth_url, state } = response.data;
    localStorage.setItem('oauth_state', state);
    window.location.href = auth_url;
  };

  // Processar callback
  const handleCallback = async (currentUrl: string) => {
    const response = await axios.get(currentUrl, {
      headers: { 'X-Tenant-ID': TENANT_ID }
    });

    const { token: newToken, user: newUser } = response.data;
    token.value = newToken;
    user.value = newUser;

    localStorage.setItem('token', newToken);
    localStorage.setItem('user', JSON.stringify(newUser));
    localStorage.removeItem('oauth_state');
  };

  // Logout
  const logout = () => {
    token.value = null;
    user.value = null;
    localStorage.removeItem('token');
    localStorage.removeItem('user');
  };

  // Axios com autenticação
  const createAuthAxios = () => {
    return axios.create({
      baseURL: API_URL,
      headers: {
        'Authorization': `Bearer ${token.value}`,
        'X-Tenant-ID': TENANT_ID
      }
    });
  };

  return {
    user,
    token,
    isAuthenticated,
    loadFromStorage,
    loginWithGoogle,
    loginWithApple,
    handleCallback,
    logout,
    createAuthAxios
  };
}
```

### 2. Login Component (`src/views/Login.vue`)

```vue
<template>
  <div class="login-container">
    <div class="login-card">
      <h1>Welcome</h1>
      <p>Sign in to continue</p>

      <div v-if="error" class="alert alert-error">
        {{ error }}
      </div>

      <div class="oauth-buttons">
        <button
          class="btn btn-google"
          @click="handleGoogleLogin"
          :disabled="loading"
        >
          <img src="/icons/google.svg" alt="Google" />
          {{ loading ? 'Redirecting...' : 'Continue with Google' }}
        </button>

        <button
          class="btn btn-apple"
          @click="handleAppleLogin"
          :disabled="loading"
        >
          <img src="/icons/apple.svg" alt="Apple" />
          {{ loading ? 'Redirecting...' : 'Continue with Apple' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useAuth } from '../composables/useAuth';

const { loginWithGoogle, loginWithApple } = useAuth();
const loading = ref(false);
const error = ref<string | null>(null);

const handleGoogleLogin = async () => {
  try {
    loading.value = true;
    error.value = null;
    await loginWithGoogle();
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to login with Google';
    loading.value = false;
  }
};

const handleAppleLogin = async () => {
  try {
    loading.value = true;
    error.value = null;
    await loginWithApple();
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to login with Apple';
    loading.value = false;
  }
};
</script>
```

---

## 🅰️ Angular 17+

### 1. Auth Service (`src/app/services/auth.service.ts`)

```typescript
import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, BehaviorSubject } from 'rxjs';
import { tap } from 'rxjs/operators';
import { environment } from '../../environments/environment';

export interface User {
  id: string;
  email: string;
  name?: string;
  picture?: string;
  roles: string[];
  oauth_provider: 'google' | 'apple';
}

export interface AuthResponse {
  token: string;
  user: User;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private readonly API_URL = environment.apiUrl;
  private readonly TENANT_ID = environment.tenantId;
  
  private userSubject = new BehaviorSubject<User | null>(null);
  public user$ = this.userSubject.asObservable();

  constructor(private http: HttpClient) {
    this.loadFromStorage();
  }

  private getHeaders(): HttpHeaders {
    return new HttpHeaders({
      'X-Tenant-ID': this.TENANT_ID
    });
  }

  private getAuthHeaders(): HttpHeaders {
    const token = this.getToken();
    return new HttpHeaders({
      'Authorization': `Bearer ${token}`,
      'X-Tenant-ID': this.TENANT_ID
    });
  }

  loadFromStorage(): void {
    const user = localStorage.getItem('user');
    if (user) {
      this.userSubject.next(JSON.parse(user));
    }
  }

  getGoogleAuthUrl(): Observable<{ auth_url: string; state: string }> {
    return this.http.get<{ auth_url: string; state: string }>(
      `${this.API_URL}/api/auth/google`,
      { headers: this.getHeaders() }
    );
  }

  getAppleAuthUrl(): Observable<{ auth_url: string; state: string }> {
    return this.http.get<{ auth_url: string; state: string }>(
      `${this.API_URL}/api/auth/apple`,
      { headers: this.getHeaders() }
    );
  }

  handleCallback(currentUrl: string): Observable<AuthResponse> {
    return this.http.get<AuthResponse>(currentUrl, {
      headers: this.getHeaders()
    }).pipe(
      tap(response => {
        localStorage.setItem('token', response.token);
        localStorage.setItem('user', JSON.stringify(response.user));
        this.userSubject.next(response.user);
      })
    );
  }

  getToken(): string | null {
    return localStorage.getItem('token');
  }

  isAuthenticated(): boolean {
    return !!this.getToken();
  }

  logout(): void {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    localStorage.removeItem('oauth_state');
    this.userSubject.next(null);
  }
}
```

### 2. Login Component (`src/app/pages/login/login.component.ts`)

```typescript
import { Component } from '@angular/core';
import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.css']
})
export class LoginComponent {
  loading = false;
  error: string | null = null;

  constructor(private authService: AuthService) {}

  loginWithGoogle(): void {
    this.loading = true;
    this.error = null;

    this.authService.getGoogleAuthUrl().subscribe({
      next: ({ auth_url, state }) => {
        localStorage.setItem('oauth_state', state);
        window.location.href = auth_url;
      },
      error: (err) => {
        this.error = err.error?.message || 'Failed to login with Google';
        this.loading = false;
      }
    });
  }

  loginWithApple(): void {
    this.loading = true;
    this.error = null;

    this.authService.getAppleAuthUrl().subscribe({
      next: ({ auth_url, state }) => {
        localStorage.setItem('oauth_state', state);
        window.location.href = auth_url;
      },
      error: (err) => {
        this.error = err.error?.message || 'Failed to login with Apple';
        this.loading = false;
      }
    });
  }
}
```

---

## 🔐 Segurança - Todas as Frameworks

### CSRF Protection

```typescript
// Sempre validar state parameter
const validateState = (): boolean => {
  const urlParams = new URLSearchParams(window.location.search);
  const receivedState = urlParams.get('state');
  const savedState = localStorage.getItem('oauth_state');
  
  if (!receivedState || !savedState || receivedState !== savedState) {
    console.error('CSRF validation failed');
    return false;
  }
  
  return true;
};
```

### Token Expiration

```typescript
// Verificar expiração do JWT
const isTokenExpired = (token: string): boolean => {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return Date.now() >= payload.exp * 1000;
  } catch {
    return true;
  }
};
```

---

**Pronto para integrar no seu frontend!** 🎉

Veja também:
- [OAUTH_SETUP.md](./OAUTH_SETUP.md) - Configuração backend
- [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) - Guia de migração
