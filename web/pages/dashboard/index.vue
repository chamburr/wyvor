<template>
  <div>
    <Heading title="Dashboard">
      Please select a server to get started. If you don't see your server here, try refreshing the
      page.
    </Heading>
    <div class="row">
      <div v-for="element in orderedGuilds" :key="element.id" class="col-12 col-md-6 col-lg-4 pb-3">
        <Card class="bg-dark h-100" body-classes="py-3 d-flex flex-column justify-content-between">
          <div class="d-flex">
            <img
              class="dashboard-icon mw-100 mh-100 mr-3 rounded-circle"
              :src="$discord.getIcon(element)"
              alt="Icon"
            />
            <div class="d-inline-flex flex-column justify-content-center text-wrap">
              <span class="h5 font-weight-bold text-wrap mb-0">
                {{ element.name }}
              </span>
            </div>
          </div>
          <div class="mt-3">
            <NuxtLink v-if="element.has_bot" :to="`/dashboard/${element.id}`">
              <BaseButton class="w-100" type="success" size="sm">
                <span>Go to Dashboard</span>
              </BaseButton>
            </NuxtLink>
            <a v-else target="_blank" :href="`/invite?guild=${element.id}`">
              <BaseButton class="w-100" type="gray" size="sm">
                <span>Add to Server</span>
              </BaseButton>
            </a>
          </div>
        </Card>
      </div>
    </div>
  </div>
</template>

<script>
import { mapGetters } from 'vuex'

export default {
  layout: 'auth',
  head: {
    title: 'Dashboard',
  },
  computed: {
    orderedGuilds() {
      return [
        ...this.guilds.filter(element => element.has_bot),
        ...this.guilds.filter(element => !element.has_bot && (element.permissions & 20) === 20),
      ]
    },
    ...mapGetters('user', ['guilds']),
  },
}
</script>

<style scoped>
.dashboard-icon {
  height: 60px;
  width: 60px;
}
</style>
