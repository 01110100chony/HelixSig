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

## Roteiro de demonstração H3/H4 (pendente)

1. Desenhar quem possui cada sender/receiver e explicar por que o coletor recebe
   até o fechamento do canal antes de fazer join dos workers.
2. Explicar por que Q=1 não determina batches de tamanho fixo: cada worker espera
   um evento e tenta obter os demais sem esperar completar B.
3. Reproduzir os testes de fila cheia e de produtor com admissão pendente;
   distinguir dropped, not_admitted e aborted usando os conjuntos de IDs.
4. Mostrar uma execução interrompida com SIGINT e seu Parquet parcial válido.
   Comparar com a saída incompleta de uma falha de escrita, onde written=0.
5. Explicar o teste de panic depois de um resultado enviado: esse ID já pertence
   ao coletor e não pode ser contado novamente como aborted. Os testes usam
   pontos de sincronização que não existem no executável de produção.

## Roteiro de demonstração H5 (pendente)

1. Localizar o commit, os hashes do corpus e o estado limpo em
   `docs/evidence/H5/campaign.json`. Distinguir as 124 rodadas de aquecimento das
   620 medições e explicar por que o smoke não comprova desempenho.
2. Comparar W=1,2,4 no centro com política block e FFI batch. Usar medianas e
   dispersão; explicar por que o resultado não demonstra escala linear.
3. Explicar a perda mediana de 89,246% no centro com drop-new. Defender por que
   throughput de resultados escritos não pode ocultar eventos descartados.
4. Mostrar onde começa e termina o microbenchmark, quais cópias são medidas e
   como todos os resultados são validados depois. Explicar por que subtrair o
   tempo do executável nativo não fornece o custo exato da FFI.
5. Defender uma conclusão limitada a esta campanha e propor um experimento
   adicional, sem afirmar que ele já foi executado. Não calcular média de p99.

Os dados e as revisões de IA não registram aprovação deste aprendizado.
