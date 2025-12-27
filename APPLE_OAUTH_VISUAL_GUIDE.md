# 🍎 Guia Visual Detalhado - Apple Sign In

## 📍 Passo 1: Acessar Apple Developer

1. **Acesse:** https://developer.apple.com/account/
2. **Faça login** com seu Apple ID
3. Se não tiver conta developer:
   - Clique em **"Enroll"** no topo
   - Escolha **Individual** ($99/ano) ou **Organization** ($99/ano)
   - Para desenvolvimento/teste, pode usar **Apple ID gratuito** (com limitações)

---

## 📍 Passo 2: Navegar até Certificates, Identifiers & Profiles

```
Apple Developer Portal
    ↓
Sidebar esquerdo: "Certificates, Identifiers & Profiles"
    ↓
Ou acesse direto: https://developer.apple.com/account/resources/identifiers/list
```

**Visual da tela:**
```
┌─────────────────────────────────────────────────────┐
│  🍎 Apple Developer                                 │
├─────────────────────────────────────────────────────┤
│  Sidebar:                                           │
│  • Overview                                         │
│  • Membership                                       │
│  ► Certificates, Identifiers & Profiles ← CLIQUE  │
│    • Certificates                                   │
│    • Identifiers        ← AQUI!                    │
│    • Profiles                                       │
│    • Devices                                        │
└─────────────────────────────────────────────────────┘
```

---

## 📍 Passo 3: Criar App ID (Identificador de Aplicação)

### 3.1 - Clicar em "Identifiers"

```
Certificates, Identifiers & Profiles
    ↓
Clique em "Identifiers" (no menu superior ou sidebar)
```

### 3.2 - Adicionar novo Identifier

```
Tela "Identifiers"
    ↓
Botão azul "+" (no canto superior esquerdo) ← CLIQUE
```

**Visual:**
```
┌──────────────────────────────────────────────────┐
│  Identifiers                         [+] ← AQUI │
├──────────────────────────────────────────────────┤
│  Lista vazia ou com identifiers existentes       │
└──────────────────────────────────────────────────┘
```

### 3.3 - Selecionar tipo "App IDs"

```
Register a new identifier
    ↓
Selecione: ⦿ App IDs  ← MARQUE ESTE
           ○ Services IDs
           ○ Pass Type IDs
           ○ Website Push IDs
           ○ Merchant IDs
    ↓
Botão "Continue" (canto superior direito)
```

### 3.4 - Escolher "App"

```
Select a type
    ↓
Selecione: ⦿ App  ← MARQUE ESTE
           ○ App Clip
    ↓
Continue
```

### 3.5 - Preencher informações do App ID

```
┌─────────────────────────────────────────────────────┐
│  Register an App ID                                 │
├─────────────────────────────────────────────────────┤
│  Description:                                       │
│  [Kong Security API]  ← SEU NOME AQUI              │
│                                                     │
│  Bundle ID:                                         │
│  ⦿ Explicit                                         │
│  [com.yourcompany.kongsecurity]  ← SEU BUNDLE ID   │
│                                                     │
│  Capabilities:                                      │
│  ☑ Sign in with Apple  ← MARQUE ESTA OPÇÃO!       │
│  ☐ Push Notifications                              │
│  ☐ Game Center                                     │
│  ... (outras capabilities)                          │
│                                                     │
│                            [Continue] [Cancel]      │
└─────────────────────────────────────────────────────┘
```

**Importante:**
- **Description:** Nome amigável (ex: "Kong Security API")
- **Bundle ID:** Identificador único (ex: `com.yourcompany.kongsecurity`)
- ✅ **MARQUE "Sign in with Apple"** na lista de Capabilities

### 3.6 - Confirmar e Registrar

```
Review and Register
    ↓
Verifique as informações
    ↓
Clique em "Register" (botão azul no canto superior direito)
```

---

## 📍 Passo 4: Criar Service ID (para Web)

### 4.1 - Voltar para Identifiers e clicar em "+"

```
Identifiers
    ↓
[+] (adicionar novo)
```

### 4.2 - Selecionar "Services IDs"

```
Register a new identifier
    ↓
Selecione: ○ App IDs
           ⦿ Services IDs  ← MARQUE ESTE
           ○ Pass Type IDs
           ○ Website Push IDs
    ↓
Continue
```

### 4.3 - Preencher informações do Service ID

```
┌─────────────────────────────────────────────────────┐
│  Register a Services ID                             │
├─────────────────────────────────────────────────────┤
│  Description:                                       │
│  [Kong Security Web]  ← NOME AMIGÁVEL              │
│                                                     │
│  Identifier:                                        │
│  [com.yourcompany.kongsecurity.web]  ← SERVICE ID  │
│                                                     │
│  ☑ Sign in with Apple  ← MARQUE!                   │
│                                                     │
│                            [Continue] [Cancel]      │
└─────────────────────────────────────────────────────┘
```

**Importante:**
- **Description:** Nome para web (ex: "Kong Security Web")
- **Identifier:** Bundle ID + ".web" (ex: `com.yourcompany.kongsecurity.web`)
- ✅ **MARQUE "Sign in with Apple"**

### 4.4 - Configurar Sign in with Apple

Após marcar "Sign in with Apple", aparecerá um botão **"Configure"**:

```
Sign in with Apple: ☑ Enabled
                    [Configure] ← CLIQUE AQUI
```

### 4.5 - Configurar Domains e URLs

```
┌──────────────────────────────────────────────────────────┐
│  Web Authentication Configuration                        │
├──────────────────────────────────────────────────────────┤
│  Primary App ID:                                         │
│  [Kong Security API ▼]  ← Selecione o App ID criado    │
│                                                          │
│  Domains and Subdomains:                                 │
│  [localhost]  ← Para desenvolvimento                    │
│  [your-domain.com]  ← Para produção                     │
│                                                          │
│  Return URLs:                                            │
│  [http://localhost:8080/api/auth/apple/callback]        │
│                                                          │
│  Para adicionar mais URLs, clique em [+]                │
│                                                          │
│                              [Done] [Cancel]             │
└──────────────────────────────────────────────────────────┘
```

**Importante:**
- **Primary App ID:** Selecione o App ID criado no Passo 3
- **Domains:** 
  - `localhost` (para testes locais)
  - `seu-dominio.com` (para produção)
- **Return URLs:**
  - `http://localhost:8080/api/auth/apple/callback` (desenvolvimento)
  - `https://seu-dominio.com/api/auth/apple/callback` (produção)

Clique em **"Done"** e depois em **"Continue"** e **"Register"**

---

## 📍 Passo 5: Criar Key (Chave para gerar Client Secret)

### 5.1 - Navegar para Keys

```
Certificates, Identifiers & Profiles
    ↓
Sidebar: Keys  ← CLIQUE AQUI
    ↓
Ou acesse: https://developer.apple.com/account/resources/authkeys/list
```

### 5.2 - Adicionar nova Key

```
Keys
    ↓
Botão [+] (no canto superior esquerdo) ← CLIQUE
```

### 5.3 - Configurar a Key

```
┌─────────────────────────────────────────────────────┐
│  Register a New Key                                 │
├─────────────────────────────────────────────────────┤
│  Key Name:                                          │
│  [Kong Security Apple Auth Key]  ← NOME DA CHAVE   │
│                                                     │
│  Key Services:                                      │
│  ☑ Sign in with Apple  ← MARQUE!                   │
│      [Configure] ← CLIQUE                           │
│  ☐ Push Notifications                              │
│  ☐ DeviceCheck                                     │
│                                                     │
│                            [Continue] [Cancel]      │
└─────────────────────────────────────────────────────┘
```

### 5.4 - Configurar Primary App ID

```
Configure Key
    ↓
Primary App ID: [Kong Security API ▼]  ← Selecione seu App ID
    ↓
[Save]
```

### 5.5 - Confirmar e Baixar

```
Confirm your key
    ↓
Revise as informações
    ↓
[Register]
```

**ATENÇÃO! Tela de Download:**
```
┌──────────────────────────────────────────────────────┐
│  ⚠️  Download Your Key                               │
├──────────────────────────────────────────────────────┤
│  This is the only time you can download your key.   │
│  Save it in a secure place.                         │
│                                                     │
│  Key ID: XYZ789ABC  ← COPIE ESTE ID!               │
│                                                     │
│  [Download]  ← BAIXE O ARQUIVO .p8                 │
│                                                     │
│  File: AuthKey_XYZ789ABC.p8                        │
└──────────────────────────────────────────────────────┘
```

**IMPORTANTE:**
1. ✅ **COPIE o Key ID** (ex: `XYZ789ABC`)
2. ✅ **BAIXE o arquivo `.p8`** (você só pode baixar UMA vez!)
3. ✅ **Salve em local seguro** (não pode ser baixado novamente!)

---

## 📍 Passo 6: Encontrar seu Team ID

```
Apple Developer Portal
    ↓
Clique no seu nome (canto superior direito)
    ↓
Selecione: "View Account" ou "Membership"
    ↓
Você verá:

┌────────────────────────────────────┐
│  Membership Information            │
├────────────────────────────────────┤
│  Team Name: Seu Nome               │
│  Team ID: ABC123DEF4  ← COPIE!    │
│  Role: Agent                       │
└────────────────────────────────────┘
```

**Team ID** parece com: `ABC123DEF4` (10 caracteres alfanuméricos)

---

## 📍 Passo 7: Gerar Client Secret (JWT)

Agora você tem:
- ✅ Team ID: `ABC123DEF4`
- ✅ Key ID: `XYZ789ABC`
- ✅ Service ID (Client ID): `com.yourcompany.kongsecurity.web`
- ✅ Arquivo .p8: `AuthKey_XYZ789ABC.p8`

### Script para gerar o JWT:

Crie um arquivo `generate_apple_secret.sh`:

```bash
#!/bin/bash

# SUAS CREDENCIAIS - SUBSTITUA AQUI!
TEAM_ID="ABC123DEF4"  # Team ID que você copiou
KEY_ID="XYZ789ABC"    # Key ID que você copiou
CLIENT_ID="com.yourcompany.kongsecurity.web"  # Service ID criado
KEY_FILE="./AuthKey_XYZ789ABC.p8"  # Caminho para arquivo .p8

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

echo ""
echo "✅ Apple Client Secret gerado com sucesso!"
echo ""
echo "Adicione ao seu .env:"
echo ""
echo "APPLE_CLIENT_ID=$CLIENT_ID"
echo "APPLE_CLIENT_SECRET=$CLIENT_SECRET"
echo "APPLE_REDIRECT_URL=http://localhost:8080/api/auth/apple/callback"
echo ""
```

### Execute o script:

```bash
# Dar permissão de execução
chmod +x generate_apple_secret.sh

# Executar
./generate_apple_secret.sh
```

### Resultado esperado:

```
✅ Apple Client Secret gerado com sucesso!

Adicione ao seu .env:

APPLE_CLIENT_ID=com.yourcompany.kongsecurity.web
APPLE_CLIENT_SECRET=eyJhbGciOiJFUzI1NiIsImtpZCI6IlhZWjc4OSISJ9.eyJpc3MiOiJBQkMxMjNERUY0IiwiaWF0IjoxNjM5...
APPLE_REDIRECT_URL=http://localhost:8080/api/auth/apple/callback
```

Copie essas linhas e cole no seu `.env`!

---

## 📍 Passo 8: Atualizar .env

```env
# Apple OAuth Configuration
APPLE_CLIENT_ID=com.yourcompany.kongsecurity.web
APPLE_CLIENT_SECRET=eyJhbGciOiJFUzI1NiIsImtpZCI6IlhZWjc4OSISJ9...
APPLE_REDIRECT_URL=http://localhost:8080/api/auth/apple/callback
```

---

## 🎯 Resumo das Credenciais Necessárias

| Credencial | Onde Encontrar | Exemplo |
|------------|----------------|---------|
| **Team ID** | Account > Membership | `ABC123DEF4` |
| **Key ID** | Keys > Sua chave criada | `XYZ789ABC` |
| **Client ID** | Identifiers > Services IDs | `com.yourcompany.kongsecurity.web` |
| **Key File** | Download após criar Key | `AuthKey_XYZ789ABC.p8` |
| **Client Secret** | Gerado pelo script JWT | `eyJhbGciOiJFUzI1Ni...` |

---

## ❓ Problemas Comuns

### "Não consigo acessar Developer Portal"

- Verifique se está usando o Apple ID correto
- Para produção, precisa ter conta paga ($99/ano)
- Para testes, pode usar conta gratuita com limitações

### "Não vejo opção 'Sign in with Apple'"

- Verifique se está criando **App ID** (não outros tipos)
- Procure na lista de **Capabilities**
- Role a página até encontrar

### "Erro ao baixar .p8"

- Arquivo só pode ser baixado UMA vez
- Se perdeu, precisa criar nova Key
- Guarde em local seguro!

### "Script não funciona"

```bash
# Verificar se OpenSSL está instalado
openssl version

# Se não estiver instalado (macOS):
brew install openssl

# Linux:
sudo apt-get install openssl
```

---

## 📱 Próximos Passos

Depois de configurar:

1. ✅ Copie as credenciais para `.env`
2. ✅ Vá para o [OAUTH_SETUP.md](./OAUTH_SETUP.md) para configurar Google
3. ✅ Teste localmente com ngrok
4. ✅ Integre no seu backend Rust

---

**Qualquer dúvida, me avise!** 🍎
