# Camada H — Engenharia Avançada (M60–M79)

Este catálogo descreve a proposta de arquitetura e o estado atual dos módulos. Os itens marcados como **implementados** são protótipos utilizáveis para experimentação e testes; não devem ser interpretados como sistemas de produção certificados.

## Estrutura

```text
src/modules/
├── mod.rs
├── m60_post_quantum_crypto.rs
├── m65_graph_jit.rs
└── m70_protocol_translator.rs
python/
├── uncertainty_estimator.py
└── test_uncertainty_estimator.py
docs/
└── camada-h.md
```

## Módulos implementados neste incremento

| ID | Módulo | Estado e limites |
|---|---|---|
| M60 | Criptografia pós-quântica | Implementa encapsulamento/decapsulamento ML-KEM-1024 (FIPS 203) via biblioteca `pqcrypto`. Não implementa assinaturas, protocolo de identidade, rotação nem armazenamento seguro de chaves. A chave secreta permanece em memória; avalie a biblioteca, o modelo de ameaça e a gestão de chaves antes de produção. |
| M65 | Planejador de fusão de grafos | Valida IDs e dependências em ordem topológica e identifica fusão MatMul+ReLU sem ramificações. Emite uma representação intermediária; **não gera código nativo, não usa LLVM e não executa JIT**. |
| M70 | Normalizador de payloads | Converte JSON fornecido via gRPC/WebSocket e quadros CAN já decodificados para um valor JSON interno. Não abre sockets, implementa transportes, Modbus ou protocolo CAN físico. |
| M75 | Estimador de incerteza | Calcula variância preditiva e entropia de Shannon sobre amostras de probabilidades. Não executa inferência nem decide ou executa ações. As entradas precisam representar saídas comparáveis do mesmo conjunto de classes. |

## Catálogo M60–M79

| ID | Nome | Linguagem proposta | Função pretendida | Estado |
|---|---|---|---|---|
| M60 | Criptografia pós-quântica | Rust | Encapsulamento de chaves pós-quântico (ML-KEM); assinatura é uma função separada (ML-DSA) | Protótipo implementado |
| M61 | Motor homomórfico | Rust/C++ | Operações permitidas diretamente sobre dados cifrados | Planejado; sem implementação |
| M62 | Orçamento de privacidade diferencial | Python | Contabilizar orçamento de privacidade (por exemplo, epsilon/delta) | Planejado; sem implementação |
| M63 | Controlador de MPC seguro | Rust | Coordenar protocolos de computação multipartidária | Planejado; sem implementação |
| M64 | Driver de enclave de hardware | Rust | Integração com ambiente de execução confiável, como SGX/SEV | Planejado; sem implementação |
| M65 | Planejador de JIT de grafos dinâmicos | Rust/LLVM | Validar e planejar fusão de operações; compilação nativa é trabalho futuro | Planejador prototipado; sem JIT nativo |
| M66 | Autoajustador de layout de tensores | Python | Explorar layouts para hierarquia de cache | Planejado; sem implementação |
| M67 | Motor de matrizes esparsas | Rust | Operações eficientes sobre matrizes esparsas | Planejado; sem implementação |
| M68 | Limitador de largura de banda de memória | Rust | Controlar contenção de memória entre tarefas | Planejado; sem implementação |
| M69 | Pipeline de fusão de kernels | Rust/CUDA | Combinar kernels compatíveis para reduzir tráfego de memória | Planejado; sem implementação |
| M70 | Tradutor de protocolos | Rust | Normalizar payloads de protocolos para formato interno | Normalizador prototipado; sem transporte |
| M71 | Despachante de micro-NPU | Rust | Distribuir inferência para aceleradores de borda | Planejado; sem implementação |
| M72 | Gateway de malha satelital | Rust | Encaminhamento tolerante a latência orbital | Planejado; sem implementação |
| M73 | Controlador cinemático de drones | Rust/ROS 2 | Integração de cinemática e navegação de atuadores | Planejado; sem implementação |
| M74 | Unidade de calibração de sensores | Rust | Estimar e corrigir desvios de sensores | Planejado; sem implementação |
| M75 | Estimador de incerteza epistêmica | Python | Medir dispersão das previsões antes de uma decisão externa | Implementado como estimador estatístico |
| M76 | Detector de desvio dinâmico | Rust | Identificar mudanças na distribuição de dados | Planejado; sem implementação |
| M77 | Simulador contrafactual | Python | Avaliar cenários alternativos definidos por modelos | Planejado; sem implementação |
| M78 | Balanceador de carga cognitiva | Rust | Distribuir recursos entre classes de tarefas | Planejado; sem implementação |
| M79 | Gerador de provas de autocorreção | Rust/Lean 4 | Integrar geração/verificação de provas formais | Planejado; sem implementação |

## Execução dos testes

Na raiz do repositório:

```sh
cargo test
python -m pip install -r python/requirements.txt
python -m unittest discover -s python -p 'test_*.py'
```

O crate `pqcrypto-kyber` fornece a implementação criptográfica; não substitua o uso da biblioteca por bytes constantes, geradores improvisados ou primitivas caseiras. A validação operacional, revisão criptográfica, auditoria de dependências e testes de interoperabilidade ainda são necessários antes de qualquer implantação sensível.
