# redis-rust

Banco de dados chave-valor em memória, em Rust, com um sistema de extensões escritas em Lua (via `mlua`).

Grupo responsável:
- Jefferson Santana
- João Rocha
- Lucas de Pinho

## Como compilar e executar

Pré-requisito: apenas o Rust instalado (`cargo`).

```bash
cargo build --release
./target/release/redis-rust
```

Ou, sem gerar o binário manualmente:

```bash
cargo run --release
```

O programa abre um prompt (`> `) e aceita três comandos, um por linha, até `EXIT`:

```
> ADD cpf_zezinho 12345678909
OK
> GET cpf_zezinho
123.456.789-09
> EXIT
```

Também funciona com a entrada vindo de um pipe (o fim da entrada encerra o programa como se fosse `EXIT`):

```bash
cat roteiro.txt | ./target/release/redis-rust
```

As extensões são carregadas do diretório `extensions/`, relativo ao diretório de onde o executável é chamado. Todo arquivo `.lua` encontrado lá é carregado no início da execução — não é preciso recompilar para adicionar, remover ou alterar uma extensão.

## Módulos e dependências

| Módulo | Responsabilidade | Depende de |
|---|---|---|
| `main.rs` | monta o banco e as extensões, inicia o laço principal | `storage`, `bridge`, `engine` |
| `engine` | laço de entrada: imprime o prompt, lê a linha, despacha o comando e escreve a resposta | `parser`, `storage`, `bridge` |
| `parser` | transforma uma linha de texto em `Command::{Add, Get, Exit}` ou erro de sintaxe | (nenhum) |
| `storage` | guarda e recupera pares chave-valor; oferece a única consulta genérica ao banco (`find_key_by_value`) | (nenhum) |
| `bridge` | carrega `extensions/`, expõe as consultas ao banco para o Lua e despacha as chamadas de `ADD`/`GET` | `mlua`, `storage` |

`mlua` só é importado em `bridge.rs`. Nenhum outro módulo sabe que existe uma VM de Lua, e nenhum deles conhece o nome de nenhuma extensão (`cpf`, `data` etc. só aparecem dentro dos próprios arquivos `.lua`).

## Protocolo de registro de extensões

Cada arquivo `.lua` dentro de `extensions/` é executado como um script independente e **deve terminar devolvendo uma tabela** com este formato:

```lua
return {
    prefix = "my_prefix_",  -- keys that start with this belong to this extension
    add = function(key, value)
        -- ADD validation/transformation. see below.
    end,
    get = function(key, value)
        -- GET formatting. see below.
    end,
}
```

- `prefix` é obrigatório: é uma string, e qualquer chave que comece com ela (`key:sub(1, #prefix) == prefix`) passa a ser tratada por essa extensão.
- `add` e `get` são **opcionais** — uma extensão pode implementar só uma das duas operações. Se não implementar, o valor passa direto (sem validação/transformação) para aquela operação.
- Uma chave sem prefixo registrado por nenhuma extensão é tratada normalmente pelo banco: `ADD` grava o valor como veio, `GET` devolve exatamente o que foi gravado.

### Assinatura de `add(key, value)`

Chamado antes de o valor ser gravado.

- `key`: a chave completa recebida no comando `ADD` (string).
- `value`: o valor recebido no comando `ADD`, ainda sem nenhuma transformação (string).
- **Retorno de sucesso**: devolva `nil` para gravar `value` exatamente como recebido, ou devolva uma `string` para gravar essa string no lugar do valor original (uso para quando a extensão precisa normalizar o dado antes de guardá-lo).
- **Retorno de erro**: chame `error("mensagem explicando o motivo", 0)`. O segundo argumento (`0`) é importante: ele diz ao Lua para **não** acrescentar informação de arquivo/linha à mensagem, então o que chega no terminal é exatamente o texto passado para `error`.

### Assinatura de `get(key, value)`

Chamado depois que o valor já foi lido do banco (o valor gravado por `add`, não o valor original digitado, se `add` tiver transformado algo).

- `key`: a chave pedida no `GET`.
- `value`: o valor atualmente gravado sob essa chave.
- **Retorno**: uma `string` com o valor formatado para exibição. `get` também pode chamar `error("mensagem", 0)` se o valor gravado não puder ser formatado (situação rara, já que `add` deveria ter barrado valores inválidos antes de chegarem a ser gravados).

### Estruturas de retorno no lado do Rust

O `bridge` traduz a chamada Lua para `Result<String, String>` — a "estrutura específica do Rust" pedida no enunciado: `Ok(valor)` carrega o valor de sucesso (o texto a gravar, no `add`, ou o texto formatado, no `get`); `Err(motivo)` carrega a explicação do erro. O motor (`engine`) usa esse `Result` diretamente: em caso de `Ok`, grava/imprime o valor; em caso de `Err`, imprime `ERRO: <motivo>` e volta ao prompt. Ou seja, todo `error("mensagem", 0)` lançado dentro do Lua vira, sem nenhuma tradução adicional, o texto depois de `ERRO:` na tela.

### Como acrescentar uma extensão nova (passo a passo)

1. Crie um arquivo `minha_extensao.lua` dentro de `extensions/`.
2. Escolha um prefixo de chave que ainda não seja usado por nenhuma outra extensão.
3. Escreva `add` (se sua extensão precisa validar/transformar o que é gravado) e/ou `get` (se precisa formatar o que é lido).
4. Termine o arquivo com `return { prefix = "...", add = ..., get = ... }`.
5. Rode o programa novamente (não precisa recompilar) — qualquer chave com aquele prefixo passa a ser tratada pela nova extensão.

## Consulta ao banco a partir de uma extensão

O `bridge` expõe duas funções Lua globais, disponíveis para qualquer extensão, dentro de `add` ou `get`:

- `db_read(key)`: devolve o valor atualmente gravado sob `key`, ou `nil` se a chave não existir.
- `db_find_key(value)`: devolve a chave que atualmente guarda exatamente este `value`, ou `nil` se nenhuma chave guardar esse valor.

Nenhuma das duas sabe o que é CPF, data ou qualquer outro prefixo — ambas trabalham sobre o banco inteiro, olhando só para o texto gravado. É `db_find_key` quem o validador de CPF usa para a regra de unicidade: ao validar `ADD cpf_x valor`, a extensão pergunta `db_find_key(valor)`; se a resposta for uma chave diferente de `cpf_x`, o CPF já existe em outro lugar e a operação falha citando essa chave.

### O problema da reentrância (ADD já em andamento durante a consulta)

O banco (`Store`) fica guardado atrás de um `Rc<RefCell<Store>>`, compartilhado entre o `engine` (que grava) e o `bridge` (que só lê, a partir das funções acima). A saída para o problema está na **ordem das operações**, não em nenhum truque de concorrência:

1. Quando chega um `ADD chave valor`, o `engine` chama `bridge::on_add`, que executa a função `add` da extensão (se houver uma para aquele prefixo). É só durante essa chamada que `db_read`/`db_find_key` podem tomar `store.borrow()` emprestado — e o empréstimo dura só a duração da própria função Lua chamada, sendo devolvido assim que ela retorna.
2. Só depois que `on_add` devolve `Ok(valor_final)` — ou seja, só depois que a extensão já terminou de consultar o banco — é que o `engine` faz `store.borrow_mut()` para gravar o novo valor.

Como o empréstimo de leitura (passo 1) e o empréstimo de escrita (passo 2) nunca acontecem ao mesmo tempo — o segundo só começa depois que o primeiro já terminou —, o `RefCell` nunca entra em conflito consigo mesmo, mesmo que a extensão consulte o banco no meio da operação de `ADD`. Não foi preciso nenhum tipo de lock, canal ou cópia do banco: a chave foi garantir, pela própria ordem do código, que leitura (dentro do Lua) e escrita (depois, no Rust) nunca se sobrepõem.

## Extensões obrigatórias

### Validador de CPF (`extensions/cpf.lua`, prefixo `cpf_`)

- `ADD`: exige exatamente 11 dígitos numéricos, sem formatação. Confere os dois dígitos verificadores pelo algoritmo padrão do CPF. Rejeita explicitamente CPFs com todos os dígitos iguais (`00000000000`, `11111111111`, ...) — esses números passam na conta do dígito verificador, mas não são CPFs válidos. Também garante unicidade: se o mesmo número já estiver gravado em outra chave, a operação falha citando essa chave (regravar o mesmo número na própria chave continua permitido).
- `GET`: formata como `000.000.000-00`.

### Formatador de data (`extensions/data.lua`, prefixo `data_`)

- `ADD`: exige o formato `aaaa-mm-dd` estrito (quatro dígitos de ano, dois de mês, dois de dia, com zero à esquerda — `2023-1-5` é rejeitado por não ter esse formato). Confere se a data existe de verdade: mês entre `01` e `12`, dia dentro do número de dias daquele mês, incluindo a regra de ano bissexto (divisível por 4, exceto século não divisível por 400). Não consulta o banco — valida só o valor recebido.
- `GET`: formata como `dd/mm/aaaa`.

## Terceira extensão

### Validador de IPv4 (`extensions/ipv4.lua`, prefixo `ip_`)

- `ADD`: exige quatro octetos separados por ponto, cada um numérico, com no máximo 3 dígitos e valor entre `0` e `255` (`256.1.1.1`, `1.2.3`, `1.2.3.4.5`, `1..2.3`, `.1.2.3`, `1.2.3.` e `1.2.3.abc` são todos rejeitados). Não consulta o banco.
- `GET`: formata como a representação binária do endereço, um byte por octeto (`192.168.1.1` → `11000000.10101000.00000001.00000001`).
- Casos de teste em `casos_teste_ip.txt`.

Por que essa extensão: ela foi escolhida porque exercita duas coisas que nem o CPF nem a data exercitam:

1. **Transformação de valor no `ADD`**: tanto o CPF quanto a data sempre devolvem `nil` no `add` (guardam o valor exatamente como foi digitado) — toda transformação delas acontece só no `GET`. O IPv4 normaliza o valor já no `ADD` (`192.168.001.001` é gravado como `192.168.1.1`, sem os zeros à esquerda), exercitando o outro caminho de retorno de `add`: devolver uma `string` para substituir o valor original.
2. **Validação de um valor com estrutura composta**: CPF e data validam uma sequência única de caracteres com regras posicionais fixas. O IPv4 precisa primeiro separar o valor em partes (por `.`) e validar cada parte individualmente (tamanho e faixa numérica) — um formato de validação diferente dos outros dois.

## Decisões de projeto em aberto

O enunciado deixou várias escolhas livres. As que tomamos, e por quê:

- **`Rc<RefCell<Store>>`, não `Arc<Mutex<Store>>`**: o programa é single-threaded (um laço lendo comandos um de cada vez, sem paralelismo). `Arc`/`Mutex` existem para compartilhar dados entre threads; aqui só precisávamos compartilhar o banco entre o `engine` e o `bridge` dentro da mesma thread, então `Rc`/`RefCell` já bastam e evitam o custo (e a complexidade) de sincronização que não seria usada.
- **Erro da extensão como `String`, não um tipo estruturado**: o Lua só sabe produzir uma mensagem de texto (`error("...", 0)`). Criar um enum de erros no Rust exigiria que o Lua codificasse de alguma forma *qual* variante escolher, o que contradiz a ideia de que o motor não conhece as extensões. Uma `String` livre é o formato mais simples que ainda atende ao pedido do enunciado ("sinalizar o insucesso e explicar o motivo").
- **Prefixo por `starts_with`, primeira extensão encontrada vence**: o registro não impede duas extensões declararem prefixos que se sobrepõem (ex.: `cpf_` e `cpf_a`). Não implementamos detecção de conflito porque o enunciado não pede unicidade de prefixo entre extensões, só que cada uma declare o seu. Na prática, quem escreve uma extensão nova deve escolher um prefixo que não colida com os existentes — é uma responsabilidade de quem estende o sistema, não do motor.
- **Ordem de carregamento de `extensions/` não é garantida**: usamos `fs::read_dir`, cuja ordem depende do sistema de arquivos. Como o item acima já evita depender de prefixos sobrepostos, essa ordem não deveria importar; se importasse, seria um sinal de que duas extensões estão competindo pelo mesmo prefixo, o que já é responsabilidade de quem as escreveu.
- **Extensão sem `add` ou sem `get` deixa o valor passar direto**: em vez de tratar "operação não implementada" como erro, tratamos como "não há nada a validar/formatar nessa direção". Isso permite, por exemplo, uma extensão que só formata a saída do `GET` sem nunca validar o `ADD`, sem forçar quem escreve a extensão a implementar as duas funções.
- **Chave é uma palavra só, sem espaços**: o enunciado diz que o valor pode ter espaços, mas não a chave. Optamos por cortar a chave no primeiro espaço da linha (tudo antes é chave, tudo depois — incluindo espaços internos — é valor), em vez de aceitar chaves com espaço e exigir algum delimitador extra.
- **Mensagens de erro nem sempre repetem o texto exato do `casos_teste.txt`**: o enunciado (seção "Interface de uso") só exige que a resposta comece com `ERRO:` e explique o motivo, não um texto específico. Por isso, por exemplo, `ip_f` (pontos duplicados), `ip_g` (ponto no início) e `ip_h` (ponto no fim) compartilham a mesma mensagem genérica de formato inválido em vez de uma frase dedicada para cada caso — o comportamento (rejeitar e não gravar) é o que importa.

