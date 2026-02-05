# Claude Explorer - Project Context

Claude Code CLI 옆에 파일 트리를 보여주는 TUI 앱. Rust + ratatui 기반.

## 프로젝트 구조

```
claude-explorer/
├── src/
│   ├── main.rs              # 진입점, 터미널 초기화, 이벤트 루프
│   ├── app.rs               # App 상태, 키 입력 핸들링, 포커스 관리
│   ├── event.rs             # 비동기 이벤트 핸들러 (키, 마우스, 리사이즈)
│   ├── terminal.rs          # PTY 관리, Claude Code 프로세스 실행
│   ├── tree/
│   │   ├── mod.rs           # FileTree 구조체, 트리 빌드/탐색 로직
│   │   └── file_node.rs     # FileNode 구조체, 파일 아이콘 매핑
│   └── ui/
│       ├── mod.rs           # draw() 함수, 레이아웃 분할
│       ├── file_tree_widget.rs   # 트리 렌더링 위젯
│       ├── terminal_widget.rs    # ANSI 파싱, 터미널 출력 위젯
│       └── help_popup.rs         # 도움말 팝업
├── Cargo.toml
├── README.md
├── LICENSE                  # MIT
└── CLAUDE.md               # 이 파일
```

## 핵심 의존성

- `ratatui` (0.29): TUI 프레임워크
- `crossterm` (0.28): 크로스플랫폼 터미널 제어
- `portable-pty` (0.8): PTY 생성 및 프로세스 관리
- `tokio` (1.42): 비동기 런타임
- `notify` (7.0): 파일시스템 감시
- `ignore` (0.4): gitignore 지원 파일 워킹
- `clap` (4.5): CLI 인자 파싱

## 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  - Terminal 초기화 (raw mode, alternate screen)             │
│  - App 생성                                                 │
│  - EventHandler 생성                                        │
│  - 메인 루프: draw() → event.next() → handle               │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│    app.rs     │    │   event.rs    │    │    ui/mod     │
│               │    │               │    │               │
│ - App struct  │    │ - Event enum  │    │ - draw()      │
│ - FileTree    │◄───│ - Tick        │    │ - Layout      │
│ - TerminalPane│    │ - Key/Mouse   │    │               │
│ - 상태 관리   │    │ - Resize      │    │               │
└───────────────┘    └───────────────┘    └───────────────┘
        │                                         │
        ▼                                         ▼
┌───────────────┐                        ┌───────────────┐
│  terminal.rs  │                        │   widgets     │
│               │                        │               │
│ - PTY 생성    │                        │ - TreeWidget  │
│ - Claude 실행 │                        │ - TermWidget  │
│ - I/O 처리    │                        │ - HelpPopup   │
└───────────────┘                        └───────────────┘
```

## 빌드 & 실행

```bash
# 개발 모드 실행
cargo run

# 릴리스 빌드
cargo build --release

# 특정 경로에서 실행
cargo run -- --path /some/project

# 테스트
cargo test

# 린트
cargo clippy

# 포맷팅
cargo fmt
```

## 코딩 컨벤션

- Rust 2021 에디션
- `cargo fmt` 스타일 준수
- 에러 처리: `anyhow::Result` 사용 (라이브러리 경계에서는 `thiserror`)
- 주석: 한글 가능, 공개 API는 영문 doc comment
- 네이밍: snake_case (함수/변수), PascalCase (타입)

## 주요 구현 포인트

### 1. PTY 통합 (terminal.rs)
- `portable-pty`로 pseudo-terminal 생성
- Claude Code를 자식 프로세스로 spawn
- 백그라운드 스레드에서 출력 읽기 (non-blocking)
- 키 입력을 PTY master로 전송

### 2. ANSI 파싱 (terminal_widget.rs)
- SGR 이스케이프 시퀀스 파싱 (색상, 볼드 등)
- ratatui Style로 변환
- 제어 문자 처리 (탭, 캐리지 리턴 등)

### 3. 파일 트리 (tree/mod.rs)
- `ignore` crate로 gitignore 지원
- 디렉토리 우선 정렬
- 선택적 확장/축소
- 검색 기능

### 4. 이벤트 루프 (event.rs)
- tokio 기반 비동기
- 250ms tick rate
- 키/마우스/리사이즈 이벤트 통합

## 테스트 전략

```bash
# 단위 테스트
cargo test

# 특정 모듈 테스트
cargo test tree::
cargo test ui::
```

## 알려진 이슈 / TODO

- [ ] PTY 리사이즈 동기화 개선
- [ ] 파일 변경 감지 (notify) 통합
- [ ] 마우스 클릭으로 파일 선택
- [ ] 설정 파일 지원 (~/.config/claude-explorer/config.toml)
- [ ] 테마 커스터마이징

## 디버깅 팁

```bash
# 로그 출력 (stderr는 TUI에 영향 안 줌)
RUST_LOG=debug cargo run 2> debug.log

# PTY 없이 트리만 테스트
cargo run -- --no-terminal  # TODO: 구현 필요
```

## 릴리스 체크리스트

1. `cargo fmt && cargo clippy`
2. `cargo test`
3. README 업데이트
4. Cargo.toml 버전 업데이트
5. `cargo publish --dry-run`
6. Git tag 생성
