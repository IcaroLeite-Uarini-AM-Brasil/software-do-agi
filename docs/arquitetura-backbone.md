# Arquitetura backbone e escala global

> **Status:** proposta arquitetural; não representa capacidade já implementada, validada ou dimensionada para produção. A meta de um bilhão de usuários simultâneos exige modelagem de carga, protótipos, testes de capacidade e validação operacional antes de qualquer compromisso.

Este documento reúne a estrutura backbone proposta para a AGI e sua extensão Edge-First. As camadas são fronteiras de responsabilidade, não módulos já existentes no repositório.

## 1. Núcleo de execução e abstração de hardware

### 1.1 Camada de abstração de SO e hardware (HAL)

Uma HAL pode expor uma interface comum para runtimes e aceleradores disponíveis na plataforma. A implementação deve detectar capacidades de forma segura e permitir fallback previsível:

- CPU: arquiteturas e extensões disponíveis no host, sem pressupor suporte a AVX-512 ou NEON.
- GPU: backends opcionais como CUDA, ROCm, Vulkan ou Metal, conforme sistema e hardware.
- NPU/TPU: adaptadores específicos para CoreML, DirectML, NNAPI ou outros runtimes suportados.
- Compilação cruzada: x86_64, ARM/ARM64 e RISC-V são alvos a avaliar; suporte efetivo depende das dependências e dos testes de cada plataforma.

A HAL deve reportar capacidades detectadas, não prometer que um backend está funcional apenas por sua presença no sistema.

### 1.2 Alocação adaptativa e execução híbrida

Um planejador pode encaminhar trabalho entre execução local, clusters de borda e serviços centrais, considerando latência, custo, capacidade disponível, privacidade, conectividade e política do usuário. Telemetria de RAM, CPU/GPU, bateria e temperatura deve ser coletada com consentimento e limites claros; decisões devem degradar de forma previsível quando sensores ou métricas não estiverem disponíveis.

## 2. Distribuição e empacotamento

- **Desktop:** daemon/serviço ou aplicação desktop com interface como Tauri; Electron é uma alternativa de interface, não requisito do núcleo.
- **Dispositivos móveis e embarcados:** bibliotecas nativas e adaptadores para Android/iOS, condicionados às APIs e restrições da plataforma.
- **Servidor e nuvem:** containers e orquestração podem padronizar serviços; Kubernetes é uma opção operacional, não uma exigência universal.
- **Isolamento:** usar sandboxing e permissões mínimas próprias de cada sistema; separar processos que manipulam modelos, dados e ferramentas.

## 3. Interface multimodal adaptativa

A arquitetura pode unificar entradas de texto, áudio, imagem, toque, gestos e controles remotos atrás de interfaces tipadas. Pré-processamento local de voz e visão reduz transmissão de dados brutos quando o dispositivo e a experiência permitirem.

A interface deve adaptar densidade, foco e navegação ao contexto: toque em celulares, navegação por foco/D-pad em TVs e painéis/atalhos em desktop. A adaptação deve preservar acessibilidade e não presumir que toda plataforma tenha câmera, microfone ou acelerador.

## 4. Segurança, privacidade e conformidade

- Processar dados sensíveis localmente sempre que viável e manter armazenamento mínimo, protegido pelos mecanismos nativos disponíveis (por exemplo, Keychain, Keystore ou TPM).
- Antes de qualquer offload, aplicar minimização de dados e controles de autorização. Anonimização não é garantia automática de irreidentificabilidade.
- Proteger transporte com TLS moderno e autenticação apropriada. E2E ou Noise exigem desenho de protocolo, gestão de chaves, rotação e revisão; citar um protocolo não implementa essas propriedades.
- Manter políticas configuráveis e revisão jurídica para LGPD, GDPR, CCPA e demais leis pertinentes. Este documento não constitui aconselhamento jurídico.

## 5. Integração, testes e atualização

CI deve testar unidades, integração, dependências e builds para alvos suportados. Emuladores não substituem testes em dispositivos reais; farms de dispositivos podem ser adicionadas conforme a matriz de plataformas. Métricas de produção precisam ser minimizadas, consentidas e revisadas quanto a privacidade.

Atualizações do aplicativo e dos modelos devem ter ciclos e mecanismos de verificação separados. Atualizações de modelos precisam de assinatura, integridade, compatibilidade, rollback e política de distribuição; deltas são uma otimização opcional, não uma garantia de menor custo total.

## 6. Proposta de escala global Edge-First

### 6.1 Topologia em níveis

```text
Dispositivos do usuário
  ├─ inferência local e pré-processamento (quando suportado e permitido)
  └─ solicitações que excedem a capacidade local
        ↓
Pontos de presença / clusters regionais de borda
  ├─ roteamento por saúde, capacidade, custo e latência observada
  └─ modelos especializados, se disponíveis
        ↓
Clusters centrais
  └─ tarefas que requerem modelos, dados ou ferramentas centralizados
```

A distribuição hipotética de carga (por exemplo, 70–80% local, 20% regional e 10% central) é apenas um cenário inicial: as faixas somam aproximadamente 100% somente sob certas interpretações e não são medições. Devem ser substituídas por perfis reais de tarefas e telemetria consentida. Dispositivos de usuários não devem virar nós de inferência para terceiros sem adesão explícita, isolamento, remuneração/termos claros e controles contra abuso.

### 6.2 Roteamento e inferência

Um roteador em níveis pode escolher entre modelos locais, de borda e centrais com base em requisitos da tarefa, qualidade, SLO, capacidade e políticas de dados. MoE, quantização e modelos menores são opções que exigem avaliações de qualidade, segurança, consumo energético e hardware.

Batching contínuo, PagedAttention e runtimes como vLLM podem elevar a utilização em cargas compatíveis. O ganho depende da arquitetura do modelo, GPU, tamanho de lote, comprimento de contexto e padrão de tráfego. Compartilhamento de KV cache requer isolamento estrito entre tenants e só deve reutilizar prefixos comprovadamente idênticos e seguros; nunca compartilhar estado privado entre usuários.

### 6.3 Comunicação e estado

QUIC/HTTP/3 ou WebTransport podem ser avaliados para streams e conexões sujeitas a mobilidade. 0-RTT pode permitir replay e não deve transportar operações não idempotentes sem proteções adequadas. Não se pode afirmar que essas opções eliminam custos de conexão ou tornam o handshake de latência zero em todos os cenários.

Serviços devem evitar estado de sessão desnecessário em processos. Estado conversacional pode permanecer no dispositivo ou ser persistido de forma cifrada e autorizada; filas particionadas, como Kafka ou Pulsar, são opções para trabalho assíncrono, com políticas de retenção, ordenação e recuperação explícitas.

### 6.4 Memória e dados distribuídos

Memória pessoal deve ter escopo de usuário, controle de acesso e cifragem. Conhecimento compartilhado exige origem, direitos de uso, retenção e remoção bem definidos. Sharding geográfico e caches hierárquicos podem ajudar, mas precisam considerar residência de dados, consistência, invalidação, privacidade e recuperação de falhas.

### 6.5 Resiliência e custos

Aplicar limites de taxa, quotas, backpressure, circuit breakers, isolamento de falhas e filas priorizadas. Sob pressão, o sistema pode oferecer respostas degradadas, adiar tarefas assíncronas ou reduzir capacidade multimodal, informando claramente as limitações. Prioridades não devem permitir que tarefas menos prioritárias sejam abandonadas indefinidamente.

## 7. Dimensionamento da meta de simultaneidade

“Um bilhão de usuários simultâneos” não é um dimensionamento executável sem definir o que simultâneo significa. Antes de estimar infraestrutura, especificar pelo menos:

1. usuários conectados versus requisições ativas e tokens por segundo;
2. distribuição de tarefas, regiões, horários de pico e taxa de cache hit;
3. modelos, qualidade, tamanho do contexto e proporção de inferência local;
4. SLOs de latência/disponibilidade e comportamento durante falhas;
5. capacidade por dispositivo/PoP, custo de energia, rede, hardware e armazenamento;
6. requisitos de privacidade, residência de dados, retenção e consentimento.

A validação deve começar por testes representativos em pequena escala, extrapolação com margens e experimentos graduais. Nenhuma porcentagem de offload, latência-alvo ou custo é garantida por este desenho.

## 8. Estado no repositório

Este texto é documentação de proposta. A presença de uma tecnologia ou camada aqui não implica que exista implementação correspondente. Consulte também o [catálogo da Camada H](camada-h.md), que registra o estado dos módulos M60–M79.
