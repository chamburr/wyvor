<template>
  <div>
    <Card
      v-for="element in logs"
      :key="element.timestamp"
      class="bg-dark mb-3"
      body-classes="overflow-scroll-x scrollbar-none px-0 mx-4 py-3"
    >
      <div class="d-flex justify-content-between text-nowrap">
        <p class="font-weight-bold mb-1 mr-3">
          {{ element.action }}
        </p>
        <p class="text-light mb-1">
          {{ new Date(element.created_at + 'Z').toUTCString().replace('GMT', 'UTC') }}
        </p>
      </div>
      <div class="d-flex justify-content-between">
        <p class="text-light small mb-0 text-nowrap mr-3">
          User: {{ $discord.getName(users[element.author]) }} ({{ element.author }})
        </p>
      </div>
    </Card>
    <div v-if="logs.length === 0" class="text-center mt-4">
      <span class="display-2">:(</span>
      <p class="h5 mt-4">No logs found.</p>
    </div>
  </div>
</template>

<script>
export default {
  layout: 'dashboard',
  async asyncData({ $axios, $fatal, $api, route }) {
    const logs = await $axios.$get(`/guilds/${route.params.id}/logs`).catch($fatal)
    const users = await $api.getUsers(logs).catch($fatal)

    return {
      logs,
      users,
    }
  },
}
</script>
