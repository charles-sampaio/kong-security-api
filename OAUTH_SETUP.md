# 🔐 Configuração OAuth - Google e Apple

## 📋 Resumo

O Kong Security API agora suporta **autenticação exclusiva via OAuth** com Google e Apple. Usuários **não podem mais** se cadastrar com email/senha - apenas através do Sign in with Google ou Sign in with Apple.

---

## 🎯 Fluxo de Autenticação

```
1. Frontend solicita URL de autorização
   GET /api/auth/google (ou /apple)
   Response: { "auth_url": "https://...", "state": "csrf_token" }

2. Frontend redireciona usuário para auth_url

3. Usuário faz login no Google/Apple

4. Provider redireciona para callback com code
   GET /api/auth/google/callback?code=xxx&state=yyy

5. Backend troca code por token, busca dados do usuário

6. Se usuário não existe, cria automaticamente

7. Retorna JWT token
   Response: { "token": "eyJhbG...", "user": {...} }

8. Frontend usa JWT em todas as requisições
```

---

## 🔧 Configuração Google OAuth

### 1. Criar Projeto no Google Cloud Console

1. Acesse [Google Cloud Console](https://console.cloud.google.com/)
2. Crie um novo projeto ou selecione um existente
3. Vá em **APIs & Services** > **Credentials**

### 2. Configurar OAuth 2.0

1. Clique em **Create Credentials** > **OAuth client ID**
2. Tipo de aplicativo: **Web application**
3. Nome: `Kong Security API`
4. **Authorized redirect URIs:**
   ```
   http://localhost:8080/api/auth/google/callback
   https://your-domain.com/api/auth/google/callback
   ```
5. Clique em **Create**
6. Copie **Client ID** e **Client Secret**

### 3. Configurar Tela de Consentimento

1. Vá em **OAuth consent screen**
2. Tipo de usuário: **External**
3. Preencha:
   - **App name**: Kong Security API
   - **User support email**: seu email
   - **Developer contact**: seu email
4. Adicione escopos:
   - `.../auth/userinfo.email`
   - `.../auth/userinfo.profile`
5. Salve

### 4. Atualizar .env

```env
GOOGLE_CLIENT_ID=123456789-abcdefghijk.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=GOCSPX-xxxxxxxxxxxxxxxxxxxxxxxx
GOOGLE_REDIRECT_URL=http://localhost:8080/api/auth/google/callback
```

---

## 🍎 Configuração Apple Sign In

### 1. Criar App ID

1. Acesse [Apple Developer](https://developer.apple.com/account/)
2. Vá em **Certificates, Identifiers & Profiles**
3. Clique em **Identifiers** > **+** (Add)
4. Selecione **App IDs** > Continue
5. Preencha:
   - **Description**: Kong Security API
   - **Bundle ID**: `com.yourcompany.kongsecurity`
6. Marque **Sign in with Apple**
7. Register

### 2. Criar Service ID

1. Identifiers > **+** (Add)
2. Selecione **Services IDs** > Continue
3. Preencha:
   - **Description**: Kong Security Web
   - **Identifier**: `com.yourcompany.kongsecurity.web`
4. Marque **Sign in with Apple**
5. **Configure**:
   - Primary App ID: selecione o App ID criado
   - **Web Domain**: `your-domain.com` (ou `localhost` para dev)
   - **Return URLs**: `http://localhost:8080/api/auth/apple/callback`
6. Continue > Register

### 3. Criar Client Secret (Key)

1. Vá em **Keys** > **+** (Add)
2. Nome: **Kong Security Apple Auth Key**
3. Marque **Sign in with Apple**
4. Configure > selecione seu Primary App ID
5. Continue > Register
6. **Baixe o arquivo `.p8`** (você só pode baixar UMA vez!)
7. Anote o **Key ID**

### 4. Gerar Client Secret JWT

Apple exige que você gere um JWT assinado como client_secret. Use este script:

```bash
# install_apple_jwt.sh
#!/bin/bash

# Suas credenciais
TEAM_ID="ABC123DEF4"  # Team ID da Apple
KEY_ID="XYZ789"       # Key ID da sua chave .p8
CLIENT_ID="com.yourcompany.kongsecurity.web"  # Service ID
KEY_FILE="path/to/AuthKey_XYZ789.p8"  # Arquivo .p8 baixado

# Gerar JWT (expira em 6 meses)
EXPIRATION=$(($(date +%s) + 15777000))

# Header
HEADER='{"alg":"ES256","kid":"'$KEY_ID'"}'
HEADER_B64=$(echo -n "$HEADER" | openssl base64 -e -A | tr -- '+/' '-_' | tr -d '=')

# Payload
PAYLOAD='{"iss":"'$TEAM_ID'","iat":'$(date +%s)',"exp":'$EXPIRATION',"aud":"https://appleid.apple.com","sub":"'$CLIENT_ID'"}'
PAYLOAD_B64=$(echo -n "$PAYLOAD" | openssl base64 -e -A | tr -- '+/' '-_' | tr -d '=')

# Assinatura
SIGNATURE=$(echo -n "$HEADER_B64.$PAYLOAD_B64" | openssl dgst -sha256 -sign "$KEY_FILE" | openssl base64 -e -A | tr -- '+/' '-_' | tr -d '=')

# JWT completo
CLIENT_SECRET="$HEADER_B64.$PAYLOAD_B64.$SIGNATURE"

echo "APPLE_CLIENT_SECRET=$CLIENT_SECRET"
```

Execute:
```bash
chmod +x generate_apple_secret.sh
./generate_apple_secret.sh
```

### 5. Atualizar .env

```env
APPLE_CLIENT_ID=com.yourcompany.kongsecurity.web
APPLE_CLIENT_SECRET=eyJhbGciOiJFUzI1NiIsImtpZCI6IlhZWjc4OSJ9.eyJpc3MiOiJBQkMxMjNERUY0...
APPLE_REDIRECT_URL=http://localhost:8080/api/auth/apple/callback
```

---

## 📱 Endpoints da API

### 1. Obter URL de Autorização do Google

```http
GET /api/auth/google
Headers: X-Tenant-ID: {tenant_id}

Response 200:
{
  "auth_url": "https://accounts.google.com/o/oauth2/v2/auth?...",
  "state": "csrf_token_12345"
}
```

**Frontend:** Redirecione o usuário para `auth_url`

### 2. Callback do Google

```http
GET /api/auth/google/callback?code=4/xxxxx&state=csrf_token_12345
Headers: X-Tenant-ID: {tenant_id}

Response 200:
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "507f1f77bcf86cd799439011",
    "email": "user@gmail.com",
    "name": "John Doe",
    "picture": "https://lh3.googleusercontent.com/...",
    "roles": ["user"]
  }
}
```

### 3. Obter URL de Autorização da Apple

```http
GET /api/auth/apple
Headers: X-Tenant-ID: {tenant_id}

Response 200:
{
  "auth_url": "https://appleid.apple.com/auth/authorize?...",
  "state": "csrf_token_67890"
}
```

### 4. Callback da Apple

```http
GET /api/auth/apple/callback?code=c_xxxxx&state=csrf_token_67890
Headers: X-Tenant-ID: {tenant_id}

Response 200:
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "507f1f77bcf86cd799439012",
    "email": "user@privaterelay.appleid.com",
    "name": null,
    "picture": null,
    "roles": ["user"]
  }
}
```

---

## 🖥️ Integração Frontend

### React Example

```typescript
import { useState } from 'react';

const LoginPage = () => {
  const [loading, setLoading] = useState(false);
  const tenantId = 'your-tenant-id';

  const loginWithGoogle = async () => {
    setLoading(true);
    
    // 1. Obter URL de autorização
    const response = await fetch('http://localhost:8080/api/auth/google', {
      headers: {
        'X-Tenant-ID': tenantId
      }
    });
    
    const { auth_url, state } = await response.json();
    
    // 2. Salvar state para verificar depois
    localStorage.setItem('oauth_state', state);
    
    // 3. Redirecionar para Google
    window.location.href = auth_url;
  };

  const loginWithApple = async () => {
    setLoading(true);
    
    const response = await fetch('http://localhost:8080/api/auth/apple', {
      headers: {
        'X-Tenant-ID': tenantId
      }
    });
    
    const { auth_url, state } = await response.json();
    localStorage.setItem('oauth_state', state);
    window.location.href = auth_url;
  };

  return (
    <div>
      <h1>Login</h1>
      <button onClick={loginWithGoogle} disabled={loading}>
        <img src="/google-icon.svg" alt="Google" />
        Sign in with Google
      </button>
      
      <button onClick={loginWithApple} disabled={loading}>
        <img src="/apple-icon.svg" alt="Apple" />
        Sign in with Apple
      </button>
    </div>
  );
};
```

### Callback Page (React)

```typescript
import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

const OAuthCallback = () => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const tenantId = 'your-tenant-id';

  useEffect(() => {
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const savedState = localStorage.getItem('oauth_state');

    // Verificar CSRF
    if (state !== savedState) {
      alert('Invalid state parameter');
      navigate('/login');
      return;
    }

    // Backend já processou o callback automaticamente
    // URL já é /api/auth/google/callback ou /api/auth/apple/callback
    // Backend retorna o token
    
    const handleCallback = async () => {
      const currentUrl = window.location.href;
      const response = await fetch(currentUrl, {
        headers: {
          'X-Tenant-ID': tenantId
        }
      });

      const { token, user } = await response.json();
      
      // Salvar token
      localStorage.setItem('token', token);
      localStorage.setItem('user', JSON.stringify(user));
      localStorage.removeItem('oauth_state');
      
      // Redirecionar para dashboard
      navigate('/dashboard');
    };

    handleCallback();
  }, []);

  return <div>Processing login...</div>;
};
```

---

## 🔒 Segurança

### CSRF Protection

- Sempre use o `state` parameter
- Verifique no callback se o state recebido == state salvo
- Estado tem validade curta (5 minutos)

### Token Validation

- Tokens JWT expiram em 2 horas (configurável)
- Use refresh tokens para renovar
- Sempre use HTTPS em produção

### Rate Limiting

- Login OAuth: 5 tentativas/minuto por IP
- Protege contra brute force

---

## 🗄️ Modelo de Dados

### User com OAuth

```rust
{
  "_id": ObjectId("..."),
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@gmail.com",
  "password": null,  // OAuth não usa senha
  "oauth_provider": "google",  // ou "apple"
  "oauth_id": "1234567890",  // Google/Apple user ID
  "name": "John Doe",
  "picture": "https://...",
  "roles": ["user"],
  "email_verified": true,  // OAuth já verificou
  "is_active": true,
  "created_at": "2025-12-23T10:00:00Z",
  "updated_at": "2025-12-23T10:00:00Z",
  "last_login": "2025-12-23T15:30:00Z"
}
```

---

## 🧪 Testando Localmente

### 1. Configurar Ngrok (para callbacks em localhost)

```bash
# Instalar ngrok
brew install ngrok

# Expor porta 8080
ngrok http 8080

# Output: https://abc123.ngrok.io -> http://localhost:8080
```

### 2. Atualizar Redirect URLs

- Google Console: `https://abc123.ngrok.io/api/auth/google/callback`
- Apple Developer: `https://abc123.ngrok.io/api/auth/apple/callback`
- .env: `GOOGLE_REDIRECT_URL=https://abc123.ngrok.io/api/auth/google/callback`

### 3. Testar

```bash
# Rodar servidor
cargo run

# Acessar no navegador
open https://abc123.ngrok.io/swagger-ui/
```

---

## 📊 Diferenças Google vs Apple

| Feature | Google | Apple |
|---------|--------|-------|
| **Email** | Real email | Pode ser proxy (privaterelay.appleid.com) |
| **Nome** | Sempre disponível | Pode não estar disponível |
| **Foto** | URL da foto | Não fornece |
| **ID Format** | Numérico | Alfanumérico |
| **Client Secret** | String fixa | JWT que expira (renovar a cada 6 meses) |
| **Email Verified** | true/false | Sempre true |

---

## 🚫 Endpoints Antigos (DESABILITADOS)

Estes endpoints foram removidos/desabilitados:

- ❌ `POST /api/auth/register` - Criar conta com senha
- ❌ `POST /api/auth/login` - Login com senha
- ❌ `POST /api/auth/password-reset/request` - Reset de senha

**Agora apenas OAuth!** 🔐

---

## 🔍 Troubleshooting

### Erro: "OAuth not configured"

- Verifique se GOOGLE_CLIENT_ID e GOOGLE_CLIENT_SECRET estão no .env
- Reinicie o servidor após atualizar .env

### Erro: "redirect_uri_mismatch"

- URL no Google Console deve ser EXATAMENTE igual ao .env
- Incluir http:// ou https://
- Sem trailing slash

### Erro: "Invalid ID token"

- Apple client secret expirou (gere novo JWT)
- Verifique se Key ID e Team ID estão corretos

### Usuário não aparece com nome (Apple)

- Apple só retorna nome na primeira vez
- Nome não é obrigatório no Apple Sign In
- Pode pedir nome manualmente no seu app

---

## 📚 Recursos

- [Google OAuth Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Apple Sign In Documentation](https://developer.apple.com/documentation/sign_in_with_apple)
- [OAuth 2.0 RFC](https://tools.ietf.org/html/rfc6749)

---

**Pronto! Autenticação 100% OAuth configurada!** 🎉
