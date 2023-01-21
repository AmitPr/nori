<script lang="ts">
  let username: String = "";
  let password: String = "";
  let remember: boolean = false;

  let loading: boolean = false;

  enum AuthAction {
    Login,
    Register,
  }

  type AuthRequest = {
    username: String;
    password: String;
    remember?: boolean;
  };

  async function authenticate(type: AuthAction) {
    loading = true;
    const request: AuthRequest = {
      username,
      password,
      remember,
    };
    switch (type) {
      case AuthAction.Login:
        {
          const res = await fetch("/api/login", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(request),
          });
          console.log(await res.json());
          loading = false;
        }
        break;
      case AuthAction.Register:
        {
          const res = await fetch("/api/register", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(request),
          });
          console.log(res.body);
          loading = false;
        }
        break;
    }
  }
</script>

<main class="w-full min-h-screen flex flex-col items-center justify-center">
  <div class="rounded-lg shadow bg-white max-w-md">
    <div class="p-4">
      <h1 class="text-2xl font-bold text-center">Login / Register</h1>
      <div class="mt-4">
        <label class="block text-sm font-bold mb-2" for="username">
          Username
        </label>
        <input
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-gray-400"
          type="text"
          id="username"
          placeholder="Username"
          bind:value={username}
        />
      </div>
      <div class="mt-2">
        <label class="block text-sm font-bold mb-2" for="password">
          Password
        </label>
        <input
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:border-gray-400"
          type="password"
          id="password"
          placeholder="Password"
          bind:value={password}
        />
      </div>
      <div class="mt-2 flex flex-row items-center space-x-1">
        <label class="inline text-sm font-bold" for="remember">
          Remember me
        </label>
        <div>
          <input
            class="border border-gray-300 rounded-lg focus:outline-none focus:border-gray-400"
            type="checkbox"
            id="remember"
            bind:checked={remember}
          />
        </div>
      </div>
      <div class="mt-4 flex flex-row justify-between">
        <button
          class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline
          disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-gray-400 disabled:hover:bg-gray-400 disabled:text-gray-600"
          type="button"
          on:click={() => authenticate(AuthAction.Login)}
          disabled={loading}
        >
          Login
        </button>
        <button
          class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline
          disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-gray-400 disabled:hover:bg-gray-400 disabled:text-gray-600"
          type="button"
          on:click={() => authenticate(AuthAction.Register)}
          disabled={loading}
        >
          Register
        </button>
      </div>
    </div>
  </div>
</main>
