# Módulo 1: ingestão e vetorização

Este é um protótipo executável da camada de captura e busca vetorial. O coletor Rust busca **somente URLs fornecidas explicitamente pelo chamador**, exige HTTPS e correspondência exata com uma lista de hosts permitidos, não segue redirecionamentos e limita concorrência, tamanho da resposta, tamanho do lote e tempo por requisição. Ele não rastreia links nem faz coleta global autônoma. Configure também regras de saída de rede no ambiente de execução; a lista de hosts não substitui controles de egress/DNS.

O `VectorStoreEngine` recebe embeddings produzidos externamente e oferece busca por cosseno em memória. Ele não chama um modelo nem gera embeddings. A matriz normalizada é armazenada em cache e invalidada quando um documento é adicionado.

## Testes

```sh
cargo test
python -m pip install -r python/requirements.txt
python -m unittest discover -s python -p 'test_*.py'
```

Não há ainda transporte IPC entre Rust e Python, persistência, treinamento contínuo, extração semântica, validação de permissões de conteúdo ou benchmark de produção. Não interprete este protótipo como uma AGI ou como prova de ingestão em escala.
