# Catálogo proposto dos módulos M10–M59

> **Status:** especificação de arquitetura. Os módulos desta lista não estão implementados nem integrados apenas por constarem no catálogo. Não usar a expressão “59 módulos integrados” ou declarar prontidão operacional sem implementação verificável, testes e evidência de integração.

Este documento registra a proposta dos módulos M10–M59. Os nomes e funções são mantidos como escopo pretendido; a coluna de estado evita confundir planejamento com funcionalidades existentes.

| ID | Nome do módulo | Linguagem proposta | Função pretendida | Estado neste repositório |
|---|---|---|---|---|
| M10 | Audio Stream VAD | Rust | Detecção de atividade de voz e segmentação de áudio | Planejado; sem implementação |
| M11 | Vision Frame Quantizer | Rust | Quantização de frames de vídeo de alta taxa de quadros | Planejado; sem implementação |
| M12 | Time-Series Anomaly | Python | Detecção de anomalias em métricas de telemetria | Planejado; sem implementação |
| M13 | Sensory Fusion Matrix | Rust | Fusão de dados multissensoriais (áudio, visão e texto) | Planejado; sem implementação |
| M14 | BCI Signal De-noiser | Rust | Filtragem de ruído para sinais de interface cérebro-computador (BCI) | Planejado; sem implementação |
| M15 | Spatial-Temporal Ingestor | Rust | Processamento de coordenadas geográficas e temporais | Planejado; sem implementação |
| M16 | HNSW Vector Indexer | Rust | Busca por vizinhos próximos em alta dimensão | Planejado; sem implementação |
| M17 | Semantic Cache | Python | Armazenamento temporário de respostas por similaridade | Planejado; sem implementação |
| M18 | Episodic Working Memory | Python | Gerenciamento de contexto de curto prazo | Planejado; sem implementação |
| M19 | WAL Journaling | Rust | Registro de alterações para recuperação | Planejado; sem implementação |
| M20 | Cross-Modal Mapper | Python | Projeção de embeddings entre modalidades, como visão e texto | Planejado; sem implementação |
| M21 | Quantization Engine | Rust | Compressão de vetores de 32 bits para 8 bits | Planejado; sem implementação |
| M22 | SAT/SMT Solver Interface | Rust | Integração com solvers formais de satisfatibilidade | Planejado; sem implementação |
| M23 | First-Order Logic Evaluator | Python | Avaliação de predicados de lógica de primeira ordem | Planejado; sem implementação |
| M24 | Causal Graph Inferrer | Python | Inferência de relações causais e análise de causa raiz | Planejado; sem implementação |
| M25 | Anti-Hallucination Guard | Python | Verificação cruzada de afirmações com fontes de dados | Planejado; sem implementação |
| M26 | Abductive Reasoning Engine | Python | Geração de hipóteses para observações incompletas | Planejado; sem implementação |
| M27 | Proof Tree Verifier | Rust | Validação de árvores de prova matemática | Planejado; sem implementação |
| M28 | Consensus Engine Raft | Rust | Consenso entre nós de um cluster | Planejado; sem implementação |
| M29 | Tool Registry | Python | Catálogo e despacho controlado de ferramentas externas | Planejado; sem implementação |
| M30 | Dynamic Task Re-scheduler | Python | Reorganização de DAGs de tarefas após falhas | Planejado; sem implementação |
| M31 | Multi-Agent Negotiator | Python | Resolução de conflitos entre agentes | Planejado; sem implementação |
| M32 | Sub-goal Failure Recovery | Python | Estratégias de recuperação para subtarefas com falha | Planejado; sem implementação |
| M33 | Swarm Intelligence Router | Rust | Distribuição de tarefas entre nós pares | Planejado; sem implementação |
| M34 | eBPF Kernel Firewall | Rust | Filtragem de tráfego/eventos no kernel, com escopo de política a definir | Planejado; sem implementação |
| M35 | Capability Sandbox | Rust | Restrição de recursos e operações por capacidades | Planejado; sem implementação |
| M36 | Cryptographic Audit Ledger | Rust | Registro encadeado por hashes para evidências de auditoria | Planejado; sem implementação |
| M37 | Prompt Injection Shield | Python | Mitigações para entradas adversariais e instruções não confiáveis | Planejado; sem implementação |
| M38 | Alignment Constraint Engine | Python | Aplicação de políticas e restrições de segurança | Planejado; sem implementação |
| M39 | Zero-Knowledge Attestation | Rust | Provas de integridade/atestado com técnicas de conhecimento zero | Planejado; sem implementação |
| M40 | ROS 2 Robotics Gateway | Rust | Integração com middleware ROS 2 | Planejado; sem implementação |
| M41 | Code Auto-Refactor Engine | Python | Propostas automatizadas de refatoração, com revisão e validação | Planejado; sem implementação |
| M42 | Synthetic Data Generator | Python | Geração de cenários sintéticos para treinamento e teste | Planejado; sem implementação |
| M43 | Auto-API Binding Builder | Python | Geração de adaptadores para APIs externas | Planejado; sem implementação |
| M44 | Edge Deployment Compiler | Rust | Compilação/empacotamento para dispositivos de borda compatíveis | Planejado; sem implementação |
| M45 | Real-Time Haptic Interface | Rust | Processamento de feedback tátil | Planejado; sem implementação |
| M46 | Model Distillation Pipeline | Python | Destilação de modelos sob critérios de qualidade definidos | Planejado; sem implementação |
| M47 | Meta-Learning Optimizer | Python | Otimização experimental de hiperparâmetros | Planejado; sem implementação |
| M48 | System Self-Healing Engine | Rust | Recuperação controlada de serviços e isolamento de falhas | Planejado; sem implementação |
| M49 | Automatic Gradient Sampler | Python | Amostragem de gradientes em treinamento federado | Planejado; sem implementação |
| M50 | Mutation Operator | Python | Geração de variações estruturais em pipelines experimentais | Planejado; sem implementação |
| M51 | P2P Mesh Discovery | Rust | Descoberta de nós pares | Planejado; sem implementação |
| M52 | Anycast BGP Router | Rust | Integração/gestão de anúncios e rotas BGP anycast | Planejado; sem implementação |
| M53 | DTN Space Mesh | Rust | Comunicação tolerante a atrasos e interrupções | Planejado; sem implementação |
| M54 | Prometheus Exporter | Rust | Exposição de métricas para coleta por Prometheus | Planejado; sem implementação |
| M55 | Shared Memory Bus | Rust | Comunicação entre processos por memória compartilhada | Planejado; sem implementação |
| M56 | High-Throughput Queue | Rust | Fila assíncrona baseada em ring buffer | Planejado; sem implementação |
| M57 | Differential Privacy Guard | Python | Mecanismos de privacidade diferencial com orçamento e análise definidos | Planejado; sem implementação |
| M58 | Context Compactor | Python | Sumarização/compactação de janelas de contexto | Planejado; sem implementação |
| M59 | Global State Synchronizer | Rust | Sincronização de estado entre regiões | Planejado; sem implementação |

## Observações de escopo e segurança

- M34 precisa definir se filtra tráfego de rede ou eventos/chamadas do sistema: eBPF é uma tecnologia de instrumentação/filtragem, não uma sandbox genérica de chamadas de sistema. Políticas de kernel exigem revisão, testes de compatibilidade e mecanismo de recuperação.
- M37 pode reduzir alguns riscos de prompt injection, mas não garante que entradas adversariais sejam detectadas ou neutralizadas. Ferramentas e dados não confiáveis exigem controles de autorização fora do prompt.
- M41 não deve alterar ou publicar código autonomamente sem revisão, testes e controles de acesso.
- M52 é uma função de rede privilegiada; qualquer implementação futura deve operar apenas em infraestrutura autorizada, com política de anúncio, validação de rotas e plano de rollback.
- M57 precisa de definição formal do mecanismo, composição e contabilidade do orçamento de privacidade; adicionar ruído sem essas garantias não equivale a privacidade diferencial.
- M24 e M25 precisam indicar incerteza, proveniência e limites dos dados. Nenhum verificador garante causalidade ou ausência de alucinações por nome de módulo.
- M54 no catálogo original combinava Prometheus com OpenTelemetry. São padrões relacionados, mas distintos; este catálogo define M54 como exposição de métricas no formato consumido pelo Prometheus. A integração OpenTelemetry, se necessária, deve ser especificada separadamente.

## Estado geral do projeto

Este catálogo é uma proposta, não evidência de implementação. Nesta branch, os componentes com código incluem o protótipo de ingestão M1, os protótipos M60, M65, M70 e M75. A proposta do Módulo 9 ainda não foi adicionada como código. A existência de componentes separados tampouco demonstra que eles estejam integrados entre si ou prontos para operação distribuída.

Antes de chamar qualquer módulo de integrado, documentar sua interface, dependências, modelo de ameaças quando aplicável, testes automatizados, comportamento de falha e evidência de integração executável.
