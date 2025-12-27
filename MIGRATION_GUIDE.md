# 🔄 Guia de Migração - OAuth Exclusivo

## ⚠️ BREAKING CHANGE

A partir desta versão, **autenticação com email/senha foi REMOVIDA**. Apenas OAuth (Google e Apple) é suportado.

---

## 📊 Impacto nas Aplicações

### ❌ Endpoints Removidos

```
POST /api/auth/register        → Use OAuth
POST /api/auth/login           → Use OAuth
POST /api/auth/password-reset  → Não aplicável
```

### ✅ Novos Endpoints

```
GET  /api/auth/google                  → Obter URL de autorização
GET  /api/auth/google/callback         → Processar login Google
GET  /api/auth/apple                   → Obter URL de autorização
GET  /api/auth/apple/callback          → Processar login Apple
```

---

## 🔧 Mudanças no Frontend

### ANTES (Email/Senha)

```typescript
// ❌ Método antigo - NÃO funciona mais
const login = async (email: string, password: string) => {
  const response = await fetch('/api/auth/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Tenant-ID': tenantId
    },
    body: JSON.stringify({ email, password })
  });
  
  const { token } = await response.json();
  localStorage.setItem('token', token);
};
```

### AGORA (OAuth)

```typescript
// ✅ Método novo - OAuth flow
const loginWithGoogle = async () => {
  // 1. Obter URL de autorização
  const response = await fetch('/api/auth/google', {
    headers: { 'X-Tenant-ID': tenantId }
  });
  
  const { auth_url, state } = await response.json();
  
  // 2. Salvar state para CSRF protection
  localStorage.setItem('oauth_state', state);
  
  // 3. Redirecionar para Google
  window.location.href = auth_url;
};

// Callback page
const handleCallback = async () => {
  const params = new URLSearchParams(window.location.search);
  const code = params.get('code');
  const state = params.get('state');
  
  // Verificar CSRF
  if (state !== localStorage.getItem('oauth_state')) {
    throw new Error('Invalid state');
  }
  
  // Backend já processou - apenas redirecionar
  const response = await fetch(window.location.href, {
    headers: { 'X-Tenant-ID': tenantId }
  });
  
  const { token, user } = await response.json();
  localStorage.setItem('token', token);
  localStorage.removeItem('oauth_state');
  
  // Ir para dashboard
  window.location.href = '/dashboard';
};
```

---

## 🗄️ Mudanças no Banco de Dados

### Modelo User Atualizado

```diff
{
  "_id": ObjectId("..."),
  "tenant_id": "uuid",
  "email": "user@gmail.com",
- "password": "hashed_password",     ❌ Removido
+ "password": null,                  ✅ Agora opcional
+ "oauth_provider": "google",        ✅ Novo campo
+ "oauth_id": "1234567890",          ✅ Novo campo
+ "name": "John Doe",                ✅ Novo campo
+ "picture": "https://...",          ✅ Novo campo
  "roles": ["user"],
+ "email_verified": true,            ✅ Sempre true para OAuth
  "is_active": true,
  "created_at": "...",
  "updated_at": "...",
  "last_login": "..."
}
```

### Migração de Usuários Existentes

```javascript
// Script MongoDB para migrar usuários antigos
db.users.updateMany(
  { 
    oauth_provider: { $exists: false } 
  },
  { 
    $set: { 
      oauth_provider: null,
      oauth_id: null,
      name: null,
      picture: null
    } 
  }
);

// Usuários antigos NÃO poderão mais fazer login!
// Eles precisarão criar nova conta via OAuth
```

---

## 🚀 Checklist de Migração

### Backend

- [x] Adicionar dependências OAuth (oauth2, reqwest, base64)
- [x] Atualizar modelo User com campos OAuth
- [x] Criar OAuthService
- [x] Criar handlers OAuth
- [x] Configurar variáveis de ambiente
- [ ] **Integrar rotas no main.rs**
- [ ] **Atualizar OpenAPI docs**
- [ ] Deprecar endpoints antigos
- [ ] Testar fluxo completo

### Frontend

- [ ] Remover formulários de login/registro com senha
- [ ] Adicionar botões "Sign in with Google" e "Sign in with Apple"
- [ ] Implementar fluxo de redirecionamento OAuth
- [ ] Criar página de callback
- [ ] Implementar validação CSRF (state parameter)
- [ ] Atualizar tratamento de erros
- [ ] Testar em ambiente de desenvolvimento
- [ ] Testar em produção

### Infraestrutura

- [ ] Obter Google OAuth credentials
- [ ] Obter Apple OAuth credentials
- [ ] Configurar redirect URLs no Google Console
- [ ] Configurar redirect URLs no Apple Developer
- [ ] Atualizar variáveis de ambiente (.env)
- [ ] Configurar HTTPS (obrigatório para OAuth)
- [ ] Testar callbacks em produção

---

## 🎨 UI/UX Recomendações

### Login Page

```html
<!-- Remove formulário antigo -->
<form> ❌
  <input type="email" />
  <input type="password" />
  <button>Login</button>
</form>

<!-- Adicione botões OAuth -->
<div class="oauth-buttons"> ✅
  <button onclick="loginWithGoogle()">
    <img src="/google-icon.svg" />
    Continue with Google
  </button>
  
  <button onclick="loginWithApple()">
    <img src="/apple-icon.svg" />
    Continue with Apple
  </button>
</div>
```

### Loading States

```typescript
const [loading, setLoading] = useState(false);
const [provider, setProvider] = useState<'google' | 'apple' | null>(null);

const loginWithGoogle = async () => {
  setLoading(true);
  setProvider('google');
  // ... OAuth flow
};

// UI
{loading && (
  <div>
    Redirecting to {provider}...
  </div>
)}
```

---

## 🔍 Troubleshooting

### "Não consigo fazer login com minha conta antiga"

**Problema:** Usuários com senha não podem migrar automaticamente.

**Solução:**
1. Faça login via OAuth (Google ou Apple)
2. Sistema criará nova conta
3. Conta antiga ficará inativa (ou delete manualmente)

### "redirect_uri_mismatch"

**Problema:** URL de callback não corresponde à configurada.

**Solução:**
- Verifique .env: `GOOGLE_REDIRECT_URL` deve ser EXATAMENTE igual ao Google Console
- Incluir protocolo (http:// ou https://)
- Sem trailing slash

### "Invalid state parameter"

**Problema:** CSRF token não corresponde.

**Solução:**
- Certifique-se de salvar `state` antes de redirecionar
- Verifique se `state` na URL == `localStorage.getItem('oauth_state')`

---

## 📚 Próximos Passos

1. **Configure OAuth credentials** (Google e Apple) - veja [OAUTH_SETUP.md](./OAUTH_SETUP.md)
2. **Atualize seu frontend** seguindo os exemplos acima
3. **Teste localmente** usando ngrok
4. **Deploy em produção** com HTTPS
5. **Comunique usuários** sobre a mudança

---

## 🤝 Suporte

Dúvidas sobre a migração? Veja:
- [OAUTH_SETUP.md](./OAUTH_SETUP.md) - Configuração detalhada
- [HOW_TO_RUN.md](./HOW_TO_RUN.md) - Como rodar o projeto
- [README.md](./README.md) - Visão geral

---

**Boa migração!** 🚀
