# Raider

Raider é uma versão inspirada no [Aider](https://aider.chat/), construída inteiramente em **Rust**. Ele funciona como um agente local de Inteligência Artificial para te ajudar a programar, criar novos arquivos, editar códigos existentes e fazer integrações automáticas com seu Git (criando branches e efetuando commits para você).

Além disso, o Raider inclui uma **Interface Web (Dashboard)** amigável embutida no próprio executável, permitindo monitorar o status do agente em tempo real e configurar os provedores de IA sem a necessidade de usar o terminal.

---

## 🚀 Funcionalidades Principais

- **100% Rust:** Backend super rápido, seguro e eficiente usando `tokio` e `axum`.
- **Múltiplos Provedores de LLM:** Suporte de fábrica para:
  - **OpenAI** (`gpt-4o`, `gpt-3.5-turbo`, etc.)
  - **Anthropic** (`claude-3.5-sonnet`, etc.)
  - **Gemini** (Google)
  - **Ollama** (Modelos rodando 100% na sua máquina local de forma privada e gratuita).
- **Integração com Git:** Usa a biblioteca `git2` para, em qualquer tarefa iniciada:
  1. Identificar seu repositório Git local.
  2. Criar uma branch segura temporária (ex: `raider-task-1234abcd`).
  3. Fazer o commit de todas as alterações assim que finalizar.
- **Motor de Edição Preciso:** Usa o formato `SEARCH/REPLACE` (Procurar e Substituir), que minimiza erros e economiza tokens na hora de modificar apenas pequenos trechos do seu código.
- **Dashboard Web Embutido:** Interface Vanilla JS + Tailwind CSS leve. Inicie agentes e assista ao log de eventos da "mente" dele rolando na tela (Polling de Status) em tempo real.

---

## 🛠️ Instalação e Execução

### Pré-requisitos
- [Rust & Cargo](https://rustup.rs/) (versão 1.70 ou superior).
- Caso queira usar modelos locais (como `llama3` ou `qwen`), [instale o Ollama](https://ollama.com/) e deixe-o rodando na sua máquina.
- As chaves de API das LLMs (OpenAI, Anthropic, Gemini) que você deseja usar.

### Compilando e Rodando

Para iniciar o servidor:

```bash
cd raider
cargo run --release
```

O Raider subirá o servidor HTTP e estará escutando na porta **3000**.
Acesse em seu navegador: **http://127.0.0.1:3000**

---

## 🖥️ Como usar a Interface Web

O Dashboard possui duas abas principais:

### 1. Settings (Configurações)
Aqui você pode colar as suas chaves de API (API Keys).
Elas ficam guardadas em memória na aplicação (não são logadas ou enviadas para lugar algum além das requisições oficiais pros modelos). Se você não usa um dos provedores, pode deixar o campo em branco.

### 2. Dashboard
É o seu painel de comando.

- **Target Directory:** O caminho _absoluto_ (ex: `/home/user/meu_projeto`) da pasta onde o código está. É essencial que essa pasta seja um repositório Git inicializado (`git init`) para que ele consiga criar a branch e commitar.
- **Target Files (Opcional):** Se você sabe em quais arquivos ele deve mexer, digite o caminho deles (ex: `src/main.rs, Cargo.toml`). Se deixar em branco, o agente fará um scanner em toda a sua pasta, **respeitando o `.gitignore`**. *(Aviso: Ler a pasta inteira pode consumir muitos tokens nas APIs pagas!)*
- **Provider & Model Name:** Escolha se quer a OpenAI, Anthropic, Gemini ou Ollama, e o nome exato do modelo (ex: `gpt-4o`, `claude-3-5-sonnet-20240620`, `llama3.1`).
- **Prompt:** O que você deseja que ele faça? (Ex: "Crie uma função para calcular Fibonacci no arquivo utils.rs" ou "Mude o CSS da página principal para um tema escuro").

Clique em **Start Agent** e veja os logs na caixa preta abaixo mostrando o passo a passo dele (Lendo arquivos -> Pensando -> Criando Branch -> Editando Arquivos -> Commitando).

---

## 🧠 Arquitetura Técnica

- **`src/main.rs`**: O ponto de entrada. Inicia o servidor Axum, o roteador e hospeda os arquivos estáticos (Frontend HTML/JS).
- **`src/state.rs`**: Gerencia o estado thread-safe (`Arc<Mutex<AppState>>`) contendo os Status de Tarefas (Pending, Thinking, Done, Error) e Logs.
- **`src/api.rs`**: As rotas HTTP (`/api/settings` e `/api/agents`). Dispara processos filhos (`tokio::spawn`) para que a aplicação não trave enquanto espera as IAs responderem.
- **`src/agent.rs`**: O motor central. Define o `System Prompt`, empacota os arquivos locais no contexto do modelo, analisa as respostas (Regex em Search/Replace Blocks) e reescreve ou cria novos arquivos no seu sistema.
- **`src/fs_ops.rs`**: Interação limpa com o sistema de arquivos via a crate `ignore` (para não ler `node_modules`, `target`, pastas escondidas).
- **`src/git_ops.rs`**: Abstração do `git2` para criar as branches temporárias e efetuar Commits autônomos assinados pelo "Raider Agent".
- **`src/llm/`**: Módulo contendo os adaptadores das APIs (com a crate `reqwest`): OpenAI, Anthropic, Gemini e o servidor local do Ollama.

---

## 💡 Dicas de Uso

- **Criação de Arquivos Novos:** Ao solicitar a criação de um novo arquivo, o Raider criará todas as pastas pai (parent directories) caso elas ainda não existam.
- **Erros do Ollama:** Se você receber um erro dizendo `Failed to send request to Ollama`, certifique-se de que o software Ollama está ativo na sua máquina (`ollama run llama3`) em `localhost:11434`.

*(Feito com amor em Rust 🦀)*
