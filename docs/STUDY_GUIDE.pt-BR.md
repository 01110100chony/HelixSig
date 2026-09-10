# Preparação pessoal para entrevista

Todos os checkpoints estão PENDENTES. Aprovação técnica não comprova aprendizado.

| Etapa | Demonstração pessoal |
|---|---|
| H0 | Calcular baseline, pico e integral manualmente; explicar tolerâncias |
| H1 | Explicar ownership, lifetime, cada cópia e por que C++ não retém ponteiros |
| H2 | Seguir um evento da entrada até Parquet e contadores |
| H3 | Desenhar filas, workers e propagação de backpressure |
| H4 | Reproduzir erro de escrita e explicar o encerramento sem deadlock |
| H5 | Defender uma conclusão, uma limitação e uma alternativa experimental |
| H6 | Reproduzir do zero, executar a demo e associar bullets a evidências |

Não apresentar funcionalidades planejadas como entregues, revisão de IA como
revisão humana, WSL2 como Linux nativo ou replay como aquisição em tempo real.

## Roteiro de demonstração H2 (pendente)

1. Gerar um corpus e executar 4101 eventos; localizar o grupo de 4096 linhas e a
   cauda de 5 no Parquet. Explicar `id % rows`, `channel_id` e `sequence`.
2. Seguir as duas cópias de amostras: corpus para `Event`, depois para o buffer
   reutilizável. Explicar por que o empréstimo no CXX não copia as amostras.
3. Comparar `--ffi event` e `--ffi batch`: os números coincidem, mas os instantes
   de conclusão e as latências podem diferir.
4. Explicar por que um resultado válido começa como `unwritten` e só muda para
   `written` após fechar e renomear o arquivo. Reconstruir as três identidades
   de contagem em um caso com NaN e em um caso de falha de escrita.
5. Diferenciar latência, duração total, histograma com cobertura limitada e
   estimativa de buffers. Nenhum desses resultados comprova desempenho de H5.

Este roteiro não registra aprovação de aprendizado; a demonstração é do usuário.
