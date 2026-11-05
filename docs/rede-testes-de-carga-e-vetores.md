# Guia de rede, testes de capacidade e armazenamento vetorial distribuído

> **Status:** proposta de arquitetura e plano de validação. Os parâmetros e metas abaixo não foram medidos neste repositório. Não aplique configurações de kernel em produção sem ensaio, revisão de segurança e plano de reversão. Execute testes de carga somente em sistemas que você possui ou para os quais tem autorização explícita.

Este documento analisa a proposta de conexões em larga escala, QUIC/WebTransport, testes de capacidade e armazenamento vetorial. As cifras de 1 milhão ou 1 bilhão de conexões são hipóteses de planejamento, não capacidade demonstrada.

## 1. Limites e ajustes do Linux

Não existe um conjunto universal de `sysctl` que torne um host capaz de manter milhões de sessões. Capacidade depende de memória por sessão na aplicação e no kernel, implementação QUIC, tráfego, NIC, CPU, filas, firewall/NAT, cgroups, limites do serviço e distribuição de carga. QUIC é executado sobre UDP; parâmetros TCP como congestion control BBR e `tcp_tw_reuse` não aceleram diretamente sessões QUIC.

### 1.1 Avaliação das configurações sugeridas

| Parâmetro/ideia | Considerações antes de usar |
|---|---|
| `fs.file-max` e `nofile` | Aumentar limites não cria memória nem capacidade. Configurar o limite efetivo do serviço (por exemplo, `LimitNOFILE` no systemd) e confirmar com o processo em execução. Dimensionar gradualmente. |
| `nf_conntrack_max` | Pode elevar significativamente o consumo de RAM. Verificar se o tráfego realmente passa por conntrack, medir uso e drops, e estabelecer limites/alertas antes de alterar. Não definir milhões por estimativa. |
| `net.ipv4.tcp_*mem`, `net.core.rmem_*`, `wmem_*` | São buffers TCP/UDP e limites, não reservas garantidas por socket. Valores grandes podem aumentar pressão de memória; medir comportamento da implementação QUIC e da NIC. |
| `netdev_max_backlog` | Uma fila enorme pode ocultar sobrecarga e aumentar latência. Ajustar somente com métricas de drops, softirqs, CPU e latência. |
| `somaxconn` | Afeta filas de listen TCP, não o número de sessões QUIC sobre UDP. |
| `ip_local_port_range` | Afeta portas locais para conexões de saída, não conexões de entrada em UDP por si só. Geradores atrás de NAT podem ter limitações adicionais. |
| `tcp_tw_reuse` | Não é uma configuração para capacidade QUIC. Evite copiar receitas antigas sem validar a versão do kernel e o caso de uso TCP. |
| BBR e `default_qdisc=fq` | Aplicam-se ao TCP e podem depender de suporte/configuração do kernel. Não atribuir seus benefícios a QUIC automaticamente; avalie o algoritmo de congestionamento QUIC implementado pela biblioteca. |
| `vm.swappiness=0`, `memlock=unlimited` | Não são otimizações universais. Desativar swap ou permitir memória bloqueada ilimitada pode piorar estabilidade e afetar o host inteiro. |
| `nproc` elevado | Não aumenta capacidade de tarefas async e pode remover proteções úteis. Dimensionar pools e limites de processo pelo workload. |

Antes de uma alteração, registrar valores originais, aplicar um grupo pequeno de mudanças em ambiente de teste, observar RAM, CPU, p99, perda e erros, e ter rollback. Preferir limites explícitos por serviço/cgroup em vez de mudanças globais sem necessidade. O arquivo `limits.conf` pode não reger serviços systemd; verificar a configuração efetiva do serviço.

## 2. QUIC e WebTransport

### 2.1 Modelo de camadas

```text
Aplicação AGI (mensagens tipadas, políticas e limites)
  └─ WebTransport (sessões HTTP/3; streams e datagramas quando suportados)
       └─ HTTP/3
            └─ QUIC (streams confiáveis independentes, controle de congestionamento,
                     migração de conexão, TLS 1.3 integrado)
                 └─ UDP/IP
```

WebTransport é uma API/protocolo de aplicação sobre HTTP/3, não uma substituição automática para autenticação, autorização, controle de fluxo ou semântica de mensagens. Streams são confiáveis e ordenados individualmente; datagramas são não confiáveis e podem ser perdidos. A perda em um stream não bloqueia a entrega ordenada de outro stream, mas os streams ainda compartilham recursos e controle de congestionamento da conexão e da rede.

QUIC integra TLS 1.3. Uma reconexão pode usar 0-RTT com segredo de retomada, mas dados 0-RTT podem ser repetidos por um atacante de rede. Usar 0-RTT apenas para operações idempotentes/replay-safe e aplicar proteção contra repetição apropriada. A primeira conexão normalmente não é 0-RTT.

Migração por Connection ID pode manter uma conexão ao mudar de rede/IP, se cliente, servidor e política de rede permitirem. Não garante continuidade sem interrupção: NATs, firewalls, balanceadores e middleboxes podem interromper ou atrasar o caminho.

### 2.2 Desenho de mensagens

Definir um protocolo de aplicação versionado, independente do transporte:

- Cada mensagem inclui versão, tipo, identificador de requisição, sequência/tempo e tamanho máximo.
- Autenticar a sessão e autorizar cada operação; não confiar em identificadores enviados pelo cliente.
- Aplicar cotas por sessão/tenant, limites de streams simultâneos, timeouts, backpressure e cancelamento.
- Reservar datagramas para informação tolerante a perda (por exemplo, amostras de mídia recentes). Comandos, texto final e eventos que exigem entrega devem usar streams confiáveis ou confirmação explícita.
- Separar canais por prioridade sem presumir que prioridade elimina contenção. Evitar dados pessoais/telemetria sem necessidade e consentimento.
- Validar tamanho, formato e frequência; rejeitar dados malformados e fechar sessões abusivas de forma controlada.

A comparação com TCP/WebSocket depende do RTT, perda, implementação, payload e intermediários. QUIC evita o bloqueio de entrega entre streams, mas não remove congestionamento nem torna RAM “mínima”. WebSocket e gRPC/HTTP/2 continuam opções adequadas em vários ambientes. Medir a experiência real antes de migrar.

### 2.3 Implementação

O exemplo de servidor fornecido é um esboço, não um servidor pronto para Internet: usa identidade autoassinada, não configura autenticação/autorização, não estabelece limites de sessão, não trata adequadamente todos os erros e não demonstra suporte de produção a 0-RTT. Para interoperabilidade, usar certificados aceitos pelos clientes e validar a implementação e versão do crate WebTransport escolhido. Não desabilitar validação TLS em clientes de carga fora de um laboratório isolado.

## 3. Plano controlado de testes de capacidade

### 3.1 Escopo e autorização

Os testes devem apontar apenas para um ambiente isolado e explicitamente autorizado. Não direcionar tráfego de carga a endereços públicos, clientes, serviços compartilhados ou terceiros. Definir dono do teste, janela, taxa máxima, contatos de emergência e critério de parada; coordenar com a equipe de rede e o provedor.

Uma meta de conexões abertas não equivale a uma taxa de requisições. Especificar sessões conectadas, conexões novas por segundo, datagramas/s, bytes/s, mensagens por sessão e fração ativa/ociosa. Começar com uma réplica e um gerador pequeno; aumentar em etapas somente quando as métricas estiverem saudáveis.

### 3.2 Sequência de validação

1. **Teste funcional:** poucas sessões; validar TLS/certificados, autenticação, streams, datagramas, encerramento e reconexão.
2. **Teste de linha de base:** medir custo por sessão idle e ativa, memória de usuário/kernel, CPU por core, pacotes/s, p50/p95/p99 e erros.
3. **Rampa gradual:** aumentar sessões e taxa de criação em patamares; manter cada patamar tempo suficiente para estabilização e observar vazamentos.
4. **Teste de soak:** manter carga representativa por janela acordada, com tráfego realista e pausas. Incluir encerramentos e reconexões distribuídos para evitar um padrão artificial.
5. **Falha controlada:** testar perda de um worker/PoP, saturação de fila e reinício apenas em ambiente de ensaio, verificando recuperação e limites.
6. **Extrapolação:** repetir com mais geradores e nós de destino; documentar a eficiência incremental. Não extrapolar linearmente sem verificar gargalos compartilhados (NAT, NIC, balanceador, CPU, memória, rede e storage).

Definir critérios de parada antes do teste: erros acima do limite acordado, p99 acima do SLO, pressão de memória, OOM, drops persistentes, saturação de NIC/CPU, impacto em serviços adjacentes ou alerta do provedor. Interromper carga automaticamente ao atingir o primeiro critério.

### 3.3 Geradores e observabilidade

A distribuição de geradores evita que um único host seja o gargalo, mas cada gerador tem limites de CPU, memória, NIC, portas/NAT e implementação de QUIC. Medir a carga do gerador e usar métricas da aplicação no servidor para contar sessões; `ss -u` conta sockets UDP locais, não sessões QUIC individuais. `slabtop` e estatísticas de NIC são sinais úteis, não substituem métricas por sessão e por processo.

Coletar, com baixa cardinalidade e sem payloads pessoais: sessões ativas, novas/encerradas por segundo, streams e datagramas, bytes/s, erros por classe, latências, uso RSS, alocações, drops de kernel/NIC, filas e CPU por core. Sincronizar relógios e registrar versões/configuração de cliente, servidor, kernel e NIC.

Ferramentas como k6 podem exigir extensões específicas para QUIC/WebTransport; confirmar manutenção, protocolo suportado e validação TLS da extensão escolhida. Um executor de taxa de chegada não representa automaticamente sessões persistentes. Para 1M de sessões é necessário um gerador que controle explicitamente o número de sessões abertas e seu ciclo de vida; validar primeiro em escala pequena. Não usar `rejectUnauthorized: false` em testes conectados a redes não isoladas.

## 4. Arquitetura lógica de armazenamento vetorial distribuído

Não existe garantia geral de p99 abaixo de 15 ms ou recall acima de 95% só por escolher IVF, HNSW, PQ ou 300 PoPs. Metas dependem de região, rede, dimensões, filtros, concorrência, hardware, consistência e configuração de busca. “100 bilhões de vetores” também não decorre automaticamente de um bilhão de pessoas: é uma hipótese de volume que precisa de política de retenção e cardinalidade por usuário.

### 4.1 Domínios de dados e caminho de consulta

```text
Cliente
  ├─ memória pessoal local (opcional, cifrada e sob controle do usuário)
  └─ API autenticada
       ├─ autorização e limite de tenant/região
       ├─ roteador de consulta e política de residência
       ├─ índice regional particionado (metadados + vetor comprimido)
       ├─ armazenamento durável de vetor/documento autorizado
       └─ catálogo de shards, versões de índice e estado de replicação

Dados coletivos autorizados → pipeline separado de consentimento, remoção de identidade,
                                validação de origem/direitos e publicação de índice
```

Separar memória privada de usuário de conhecimento coletivo. Não criar um grafo coletivo a partir de conversas pessoais sem base legal, consentimento e processo de minimização/anonimização validado. Cifrar dados em trânsito e repouso; isolamento lógico de namespace não substitui autorização e testes contra acesso cruzado.

### 4.2 Particionamento, replicação e recuperação

Uma chave de partição deve incluir tenant/escopo e região/residência quando aplicável. Um hash consistente ou catálogo de shards facilita rebalanceamento; `hash(user_id) % N` simples pode exigir movimentação ampla ao mudar N. Evitar expor IDs pessoais em logs e nomes de shard. Definir separadamente:

- replicação e RPO/RTO por classe de dados;
- consistência de atualizações e remoções;
- rebalanceamento e recuperação de shard;
- autorização antes da consulta e filtragem por tenant;
- expiração, exportação e apagamento verificável;
- versionamento de modelo/embedding para reindexação compatível.

### 4.3 Índices e compressão

Para um vetor de 1536 dimensões FP32, o payload bruto é 1536 × 4 = 6144 bytes (6 KiB). Em 100 bilhões de vetores, são cerca de 614,4 PB decimais (ou aproximadamente 546 PiB), antes de índices, metadados, réplicas, logs e espaço operacional. Um código PQ de 64 bytes para cada vetor corresponderia a 6,4 PB decimais apenas para os códigos; não inclui centroides, IDs, metadados, grafo HNSW, réplicas ou buffers.

PQ/SQ é uma troca entre memória, latência e qualidade. Não é “sem perda significativa” por definição: recall precisa ser medido em dados representativos, com consultas e filtros reais. IVF limita candidatos por listas examinadas; HNSW adiciona custo de memória e manutenção. Usar HNSW dentro de cada partição IVF pode ser uma opção, mas não garante complexidade logarítmica global nem p99 predeterminado.

Uma estratégia inicial pode manter dados canônicos duráveis fora do índice aproximado e reconstruir índices por partição. Escolher tecnologia e layout somente após benchmark com cardinalidade, dimensão, filtros, taxa de atualização, concorrência, requisitos de remoção e orçamento reais.

### 4.4 Plano de benchmark do banco

- Criar conjunto de avaliação representativo e permitido, com consultas e ground truth.
- Medir recall@k/ndcg, p50/p95/p99, throughput, custo de memória por vetor, build/rebuild, ingestão e remoção.
- Repetir com filtros de tenant/região, concorrência, cache frio/quente e falha de shard.
- Incluir réplicas, metadados, índices e overhead do serviço nos cálculos de capacidade.
- Não publicar dados de clientes nem reutilizar embeddings entre tenants por cache sem prova de isolamento e autorização.

## 5. Critérios para uma prova de conceito

1. Definir SLOs e perfil de carga mensuráveis; esclarecer sessões conectadas versus atividade.
2. Implementar um protótipo de transporte com autenticação, limites e observabilidade antes de ajustar o kernel.
3. Validar numa rede de laboratório e avançar por etapas com critérios de parada.
4. Comparar transporte e índice com baseline, dados autorizados e configuração reproduzível.
5. Registrar resultados como medições do ambiente testado, nunca como capacidade universal.

Este guia é documentação de projeto; não adiciona configuração executável de kernel, gerador de tráfego nem banco distribuído ao repositório.
